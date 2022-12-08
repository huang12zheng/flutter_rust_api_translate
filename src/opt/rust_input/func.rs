use super::*;

mod api {

    use super::*;

    pub fn get_fns(file: &File) -> Vec<ItemFn> {
        let mut src_fns = extract_fns_from_file(file);
        src_fns.extend(extract_methods_from_file(file));
        src_fns
    }
    // IrTypeImplTrait' raw data sort in [`convert_impl_trait_to_bound`]
    pub fn get_sig_args(file: &File) -> IrImplTraitPool {
        let src_fns = get_fns(file);
        src_fns
            .into_iter()
            .flat_map(parse_function)
            .unique()
            .collect()
    }

    fn extract_fns_from_file(file: &File) -> Vec<ItemFn> {
        let mut src_fns = Vec::new();

        for item in file.items.iter() {
            if let Item::Fn(ref item_fn) = item {
                if let Visibility::Public(_) = &item_fn.vis {
                    src_fns.push(item_fn.clone());
                }
            }
        }

        src_fns
    }

    fn extract_methods_from_file(file: &File) -> Vec<ItemFn> {
        let mut src_fns = Vec::new();
        for item in file.items.iter() {
            if let Item::Impl(ref item_impl) = item {
                for item in &item_impl.items {
                    if let ImplItem::Method(item_method) = item {
                        if let Visibility::Public(_) = &item_method.vis {
                            let f = item_method_to_function(item_impl, item_method)
                                .expect("item implementation is unsupported");
                            src_fns.push(f);
                        }
                    }
                }
            }
        }

        src_fns
    }

    // Converts an item implementation (something like fn(&self, ...)) into a function where `&self` is a named parameter to `&Self`
    // use by `extract_methods_from_file`
    fn item_method_to_function(
        item_impl: &ItemImpl,
        item_method: &ImplItemMethod,
    ) -> Option<ItemFn> {
        #[derive(Debug)]
        pub struct FunctionName {
            actual_name: String,
            method_info: MethodInfo,
        }
        impl FunctionName {
            pub fn new(name: &str, method_info: MethodInfo) -> FunctionName {
                FunctionName {
                    actual_name: name.to_string(),
                    method_info,
                }
            }
            pub fn serialize(&self) -> String {
                const STATIC_METHOD_MARKER: &str = "__static_method__";
                const METHOD_MARKER: &str = "__method__";
                fn mark_as_static_method(s: &str, struct_name: &str) -> String {
                    format!("{}{}{}", s, STATIC_METHOD_MARKER, struct_name)
                }
                fn mark_as_non_static_method(s: &str, struct_name: &str) -> String {
                    format!("{}{}{}", s, METHOD_MARKER, struct_name)
                }
                match &self.method_info {
                    MethodInfo::Not => self.actual_name.clone(),
                    MethodInfo::Static { struct_name } => {
                        mark_as_static_method(&self.actual_name, struct_name)
                    }
                    MethodInfo::NonStatic { struct_name } => {
                        mark_as_non_static_method(&self.actual_name, struct_name)
                    }
                }
            }
        }
        #[derive(Debug)]
        pub enum MethodInfo {
            #[allow(dead_code)]
            Not,
            Static {
                struct_name: String,
            },
            NonStatic {
                struct_name: String,
            },
        }

        if let Type::Path(p) = item_impl.self_ty.as_ref() {
            let struct_name = p.path.segments.first().unwrap().ident.to_string();

            let span = item_method.sig.ident.span();
            let is_static_method = {
                let Signature { inputs, .. } = &item_method.sig;
                {
                    !matches!(inputs.first(), Some(FnArg::Receiver(..)))
                }
            };
            let method_name = if is_static_method {
                let self_type = {
                    let ItemImpl { self_ty, .. } = item_impl;
                    if let Type::Path(TypePath { qself: _, path }) = &**self_ty {
                        if let Some(PathSegment {
                            ident,
                            arguments: _,
                        }) = path.segments.first()
                        {
                            Some(ident.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                Ident::new(
                    &FunctionName::new(
                        &item_method.sig.ident.to_string(),
                        MethodInfo::Static {
                            struct_name: self_type.unwrap(),
                        },
                    )
                    .serialize(),
                    span,
                )
            } else {
                Ident::new(
                    &FunctionName::new(
                        &item_method.sig.ident.to_string(),
                        MethodInfo::NonStatic {
                            struct_name: struct_name.clone(),
                        },
                    )
                    .serialize(),
                    span,
                )
            };

            Some(ItemFn {
                attrs: vec![],
                vis: item_method.vis.clone(),
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: item_method.sig.fn_token,
                    ident: method_name,
                    generics: item_method.sig.generics.clone(),
                    paren_token: item_method.sig.paren_token,
                    inputs: item_method
                        .sig
                        .inputs
                        .iter()
                        .map(|input| {
                            if let FnArg::Receiver(Receiver { mutability, .. }) = input {
                                let mut segments = Punctuated::new();
                                segments.push(PathSegment {
                                    ident: Ident::new(struct_name.as_str(), span),
                                    arguments: PathArguments::None,
                                });
                                if mutability.is_some() {
                                    panic!("mutable methods are unsupported for safety reasons");
                                }
                                FnArg::Typed(PatType {
                                    attrs: vec![],
                                    pat: Box::new(Pat::Ident(PatIdent {
                                        attrs: vec![],
                                        by_ref: Some(syn::token::Ref { span }),
                                        mutability: *mutability,
                                        ident: Ident::new("that", span),
                                        subpat: None,
                                    })),
                                    colon_token: Colon { spans: [span] },
                                    ty: Box::new(Type::Path(TypePath {
                                        qself: None,
                                        path: Path {
                                            leading_colon: None,
                                            segments,
                                        },
                                    })),
                                })
                            } else {
                                input.clone()
                            }
                        })
                        .collect::<Punctuated<_, _>>(),
                    variadic: None,
                    output: item_method.sig.output.clone(),
                },
                block: Box::new(item_method.block.clone()),
            })
        } else {
            None
        }
    }
}
pub use api::*;
mod parse_args {
    use log::debug;

    use super::*;
    pub fn parse_function(func: ItemFn) -> Vec<IrTypeImplTrait> {
        debug!("parse_function function name: {:?}", func.sig.ident);

        let sig = func.sig;

        let inputs = sig
            .inputs
            .into_iter()
            .filter_map(|sig_input| {
                if let FnArg::Typed(ref pat_type) = sig_input {
                    Some(*pat_type.ty.clone())
                } else {
                    None
                }
            })
            .collect();
        let output = match sig.output {
            ReturnType::Type(_, ty) => Some(*ty),
            ReturnType::Default => None,
        };
        let mut sig_args: Vec<Type> = inputs;
        sig_args.extend(output.into_iter().collect::<Vec<Type>>());
        sig_args
            .iter()
            .filter_map(try_parse_fn_arg_type)
            .map(convert_impl_trait_to_bound)
            .collect()
    }
    /// Attempts to parse the type from an argument of a function signature. There is a special
    /// case for top-level `StreamSink` types.
    fn try_parse_fn_arg_type(ty: &syn::Type) -> Option<TypeImplTrait> {
        match ty {
            // syn::Type::Array(_) |
            syn::Type::ImplTrait(input) => {
                Some(input.clone())
                // Some(IrFuncArg::Type(self.type_parser.parse_type(ty)))
            }
            _ => None,
        }
    }
    #[derive(Clone, Eq, Hash, PartialEq, Debug)]
    pub struct IrTypeImplTrait {
        pub trait_bounds: Vec<String>,
    }
    pub fn convert_impl_trait_to_bound(type_impl_trait: TypeImplTrait) -> IrTypeImplTrait {
        let mut raw: Vec<String> = type_impl_trait
            .bounds
            .iter()
            .filter_map(|e| match e {
                TypeParamBound::Trait(t) => {
                    Some(t.path.segments.first().unwrap().ident.clone().to_string())
                }
                TypeParamBound::Lifetime(_) => None,
            })
            // .sorted()
            .collect();
        raw.sort();

        IrTypeImplTrait { trait_bounds: raw }
    }
}

pub use parse_args::*;
