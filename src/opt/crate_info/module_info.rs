use super::*;

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("visibility", &self.visibility)
            .field("module_path", &self.module_path)
            .field("file_path", &self.file_path)
            .field("source", &"omitted")
            .field("scope", &self.scope)
            .finish()
    }
}

impl Module {
    pub fn resolve(&mut self) {
        self.resolve_modules();
        // self.resolve_imports();
    }

    /// Maps out modules, structs and enums within the scope of this module
    fn resolve_modules(&mut self) {
        let mut scope_modules = Vec::new();
        let mut scope_structs = Vec::new();
        let mut scope_enums = Vec::new();
        let mut scope_impls = Vec::new();

        let items = match self.source.as_ref().unwrap() {
            ModuleSource::File(file) => &file.items,
            ModuleSource::ModuleInFile(items) => items,
        };

        for item in items.iter() {
            match item {
                syn::Item::Struct(item_struct) => {
                    let (idents, mirror) = get_ident(&item_struct.ident, &item_struct.attrs);

                    scope_impls.extend(get_impl_trait_from_attrs(
                        &item_struct.ident,
                        &item_struct.attrs,
                        matches!(item_struct.vis, syn::Visibility::Public(_)),
                    ));
                    scope_structs.extend(idents.into_iter().map(|ident| {
                        let ident_str = ident.to_string();
                        Struct {
                            ident,
                            src: item_struct.clone(),
                            visibility: syn_vis_to_visibility(&item_struct.vis),
                            path: {
                                let mut path = self.module_path.clone();
                                path.push(ident_str);
                                path
                            },
                            mirror,
                        }
                    }));
                }
                syn::Item::Enum(item_enum) => {
                    let (idents, mirror) = get_ident(&item_enum.ident, &item_enum.attrs);
                    scope_impls.extend(get_impl_trait_from_attrs(
                        &item_enum.ident,
                        &item_enum.attrs,
                        matches!(item_enum.vis, syn::Visibility::Public(_)),
                    ));

                    scope_enums.extend(idents.into_iter().map(|ident| {
                        let ident_str = ident.to_string();
                        Enum {
                            ident,
                            src: item_enum.clone(),
                            visibility: syn_vis_to_visibility(&item_enum.vis),
                            path: {
                                let mut path = self.module_path.clone();
                                path.push(ident_str);
                                path
                            },
                            mirror,
                        }
                    }));
                }
                syn::Item::Impl(item_impl) => {
                    if let Some((_b, ref path, _f)) = item_impl.trait_ {
                        // To rule out segments[0].arguments have values
                        if let Some(trait_) = path.get_ident() {
                            if let Type::Path(ref type_path) = *(item_impl.self_ty) {
                                scope_impls.push(Impl {
                                    self_ty: type_path.path.get_ident().unwrap().to_owned(),
                                    trait_: trait_.to_owned(),
                                })
                            }
                        }
                    }
                }
                syn::Item::Mod(item_mod) => {
                    let ident = item_mod.ident.clone();

                    let mut module_path = self.module_path.clone();
                    module_path.push(ident.to_string());

                    scope_modules.push(match &item_mod.content {
                        Some(content) => {
                            let mut child_module = Module {
                                visibility: syn_vis_to_visibility(&item_mod.vis),
                                file_path: self.file_path.clone(),
                                module_path,
                                source: Some(ModuleSource::ModuleInFile(content.1.clone())),
                                scope: None,
                            };

                            child_module.resolve();

                            child_module
                        }
                        None => {
                            let file_path =
                                get_module_file_path(ident.to_string(), &self.file_path);

                            match file_path {
                                Ok(file_path) => {
                                    let source = {
                                        let source_rust_content =
                                            fs::read_to_string(&file_path).unwrap();
                                        debug!("Trying to parse {:?}", file_path);
                                        Some(ModuleSource::File(
                                            syn::parse_file(&source_rust_content).unwrap(),
                                        ))
                                    };
                                    let mut child_module = Module {
                                        visibility: syn_vis_to_visibility(&item_mod.vis),
                                        file_path,
                                        module_path,
                                        source,
                                        scope: None,
                                    };

                                    child_module.resolve();
                                    child_module
                                }
                                Err(tried) => {
                                    warn!(
                                        "Skipping unresolvable module {} (tried {})",
                                        &ident,
                                        tried
                                            .into_iter()
                                            .map(|it| it.to_string_lossy().to_string())
                                            .fold(String::new(), |mut a, b| {
                                                a.push_str(&b);
                                                a.push_str(", ");
                                                a
                                            })
                                    );
                                    continue;
                                }
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        self.scope = Some(ModuleScope {
            modules: scope_modules,
            enums: scope_enums,
            structs: scope_structs,
            impls: scope_impls,
        });
    }

    pub fn collect_structs<'a>(&'a self, container: &mut HashMap<String, &'a Struct>) {
        let scope = self.scope.as_ref().unwrap();
        for scope_struct in &scope.structs {
            container.insert(scope_struct.ident.to_string(), scope_struct);
        }
        for scope_module in &scope.modules {
            scope_module.collect_structs(container);
        }
    }

    pub fn collect_structs_to_pool(&self) -> HashMap<String, &Struct> {
        let mut ans = HashMap::new();
        self.collect_structs(&mut ans);
        ans
    }

    pub fn collect_enums<'a>(&'a self, container: &mut HashMap<String, &'a Enum>) {
        let scope = self.scope.as_ref().unwrap();
        for scope_enum in &scope.enums {
            container.insert(scope_enum.ident.to_string(), scope_enum);
        }
        for scope_module in &scope.modules {
            scope_module.collect_enums(container);
        }
    }

    pub fn collect_enums_to_pool(&self) -> HashMap<String, &Enum> {
        let mut ans = HashMap::new();
        self.collect_enums(&mut ans);
        ans
    }

    pub fn collect_impls(&self, container: &mut HashMap<String, Vec<Impl>>) {
        let scope = self.scope.as_ref().unwrap();
        for scope_impl in &scope.impls {
            let k = scope_impl.trait_.to_string();
            let v = container.get_mut(&k);
            if let Some(l) = v {
                l.push(scope_impl.to_owned());
            } else {
                container.insert(k, vec![scope_impl.to_owned()]);
            }
        }
        for scope_module in &scope.modules {
            scope_module.collect_impls(container);
        }
    }
    // impl TA for A => ("TA", [A,..])
    pub fn collect_impls_to_pool(&self) -> HashMap<String, Vec<Impl>> {
        let mut ans = HashMap::new();
        self.collect_impls(&mut ans);
        for (_, v) in ans.iter_mut() {
            v.sort();
        }
        ans
    }
}
