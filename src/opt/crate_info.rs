/// source from https://github.com/fzyzcjy/flutter_rust_bridge/blob/5e4e8f5b04a708793b2517bc9744c8a6993b9d6d/frb_codegen/src/source_graph.rs
pub use super::*;
use cargo_metadata::MetadataCommand;
use log::{debug, warn};
use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};
use syn::{Attribute, Ident, ItemEnum, ItemStruct, Type, __private::quote::__private::TokenTree};
mod module_info;

mod intersection;
mod remove_dependencies;
pub use intersection::intersection_bound_trait_to_object_pool;
pub(crate) use remove_dependencies::*;

/// Represents a crate, including a map of its modules, imports, structs and
/// enums.
#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub manifest_path: PathBuf,
    pub root_src_file: PathBuf,
    pub root_module: Module,
}

/// set [self.scope] when recursive call [resolve]
/// look module_info.rs please
#[derive(Clone)]
pub struct Module {
    pub visibility: Visibility,
    pub file_path: PathBuf,
    pub module_path: Vec<String>,
    pub source: Option<ModuleSource>,
    pub scope: Option<ModuleScope>,
}

#[derive(Debug, Clone)]
pub enum ModuleSource {
    File(syn::File),
    ModuleInFile(Vec<syn::Item>),
}

#[derive(Debug, Clone)]
pub struct ModuleScope {
    pub modules: Vec<Module>,
    pub enums: Vec<Enum>,
    pub impls: Vec<Impl>,
    pub structs: Vec<Struct>,
}

impl Crate {
    pub fn new(manifest_path: &str) -> Self {
        let mut result = Crate::new_without_resolve(manifest_path);
        result.resolve();
        result
    }
    pub fn new_without_resolve(manifest_path: &str) -> Self {
        let (name, root_src_file) = {
            let mut cmd = MetadataCommand::new();
            cmd.manifest_path(manifest_path);

            let metadata = cmd.exec().unwrap();

            let root_package = metadata.root_package().unwrap();
            let root_src_file = {
                let lib_file = root_package
                    .manifest_path
                    .parent()
                    .unwrap()
                    .join("src/lib.rs");
                let main_file = root_package
                    .manifest_path
                    .parent()
                    .unwrap()
                    .join("src/main.rs");

                if lib_file.exists() {
                    fs::canonicalize(lib_file).unwrap()
                } else if main_file.exists() {
                    fs::canonicalize(main_file).unwrap()
                } else {
                    panic!("No src/lib.rs or src/main.rs found for this Cargo.toml file");
                }
            };
            (root_package.name.to_owned(), root_src_file)
        };
        let source_rust_content = fs::read_to_string(&root_src_file).unwrap();
        let file_ast = syn::parse_file(&source_rust_content).unwrap();

        Crate {
            name,
            manifest_path: fs::canonicalize(manifest_path).unwrap(),
            root_src_file: root_src_file.clone(),
            root_module: Module {
                visibility: Visibility::Public,
                file_path: root_src_file,
                module_path: vec!["crate".to_string()],
                source: Some(ModuleSource::File(file_ast)),
                scope: None,
            },
        }
    }

    /// Create a map of the modules for this crate
    pub fn resolve(&mut self) {
        self.root_module.resolve();
    }
}

/// Mirrors syn::Visibility, but can be created without a token
#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Crate,
    Restricted, // Not supported
    Inherited,  // Usually means private
}

fn syn_vis_to_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Crate(_) => Visibility::Crate,
        syn::Visibility::Restricted(_) => Visibility::Restricted,
        syn::Visibility::Inherited => Visibility::Inherited,
    }
}

#[derive(Clone)]
pub struct Struct {
    pub ident: Ident,
    pub src: ItemStruct,
    pub visibility: Visibility,
    pub path: Vec<String>,
    pub mirror: bool,
}

impl Debug for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Struct")
            .field("ident", &self.ident)
            .field("src", &"omitted")
            .field("visibility", &self.visibility)
            .field("path", &self.path)
            .field("mirror", &self.mirror)
            .finish()
    }
}

#[derive(Clone)]
pub struct Enum {
    pub ident: Ident,
    pub src: ItemEnum,
    pub visibility: Visibility,
    pub path: Vec<String>,
    pub mirror: bool,
}

impl Debug for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("ident", &self.ident)
            .field("src", &"omitted")
            .field("visibility", &self.visibility)
            .field("path", &self.path)
            .field("mirror", &self.mirror)
            .finish()
    }
}

/// This represents `impl {trait} for {self_ty}`
#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub struct Impl {
    pub self_ty: Ident,
    pub trait_: Ident,
}

/// Get a struct or enum ident, possibly remapped by a mirror marker
fn get_ident(ident: &Ident, _attrs: &[Attribute]) -> (Vec<Ident>, bool) {
    (vec![ident.clone()], false)
}

fn try_get_module_file_path(
    folder_path: &Path,
    module_name: &str,
    tried: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    let file_path = folder_path.join(module_name).with_extension("rs");
    if file_path.exists() {
        return Some(file_path);
    }
    tried.push(file_path);

    let file_path = folder_path.join(module_name).join("mod.rs");
    if file_path.exists() {
        return Some(file_path);
    }
    tried.push(file_path);

    None
}

fn get_module_file_path(
    module_name: String,
    parent_module_file_path: &Path,
) -> core::result::Result<PathBuf, Vec<PathBuf>> {
    let mut tried = Vec::new();

    if let Some(file_path) = try_get_module_file_path(
        parent_module_file_path.parent().unwrap(),
        &module_name,
        &mut tried,
    ) {
        return Ok(file_path);
    }
    if let Some(file_path) = try_get_module_file_path(
        &parent_module_file_path.with_extension(""),
        &module_name,
        &mut tried,
    ) {
        return Ok(file_path);
    }
    Err(tried)
}

fn get_impl_trait_from_attrs(self_ty: &Ident, attrs: &[Attribute], vis: bool) -> Vec<Impl> {
    if vis {
        attrs
            .iter()
            .flat_map(|a| a.tokens.clone().into_iter())
            .filter_map(|tt_a| {
                if let TokenTree::Group(g) = tt_a {
                    Some(g.stream().into_iter())
                } else {
                    None
                }
            })
            .flatten()
            .filter_map(|tt_g| {
                if let TokenTree::Ident(trait_) = tt_g {
                    Some(Impl {
                        self_ty: self_ty.clone(),
                        trait_,
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    }
}
