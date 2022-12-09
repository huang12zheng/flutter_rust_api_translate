mod file;
pub(crate) use file::*;
mod crate_info;
use crate_info::*;
mod customize;
// use customize::*;
mod generate;
use derive_builder::Builder;
// use generate::*;
mod rust_input;
use lib_flutter_rust_bridge_codegen::{frb_codegen, get_symbols_if_no_duplicates};
use rust_input::*;

pub(crate) use convert_case::{Case, Casing};
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::fs;
pub(crate) use std::hash::Hash;
pub(crate) use std::iter::FromIterator;
pub(crate) use syn::{punctuated::Punctuated, token::Colon, *};

pub use lib_flutter_rust_bridge_codegen::Opts;
const BOUND_PATH: &str = "src/bridge_generated_bound.rs";
#[derive(Builder)]
pub struct OptArray {
    pub bound_oject_pool: HashMap<Vec<String>, HashSet<String>>,
    pub configs: Vec<Opts>,
    pub crate_info: Crate,
    // pub irs: Vec<IrFile>,
    pub root_src_file: String,
    pub trait_to_impl_pool: TraitToImplPool,
}

impl OptArray {
    pub fn new_with_remove_translate(configs: &[Opts]) -> Self {
        let crate_info = Crate::new_without_resolve(&configs[0].manifest_path);
        let root_src_file = crate_info.root_src_file.to_str().unwrap().to_owned();
        remove_dependencies(configs, &root_src_file);
        // new_without_resolve had set ast!!
        let mut crate_info = Crate::new_without_resolve(&configs[0].manifest_path);
        crate_info.resolve();

        let trait_to_impl_pool = crate_info.root_module.collect_impls_to_pool();
        let configs = configs.to_owned();
        let ir_type_impl_traits_pool = configs.collect_irs().ir_type_impl_traits;
        // example:
        // Debug -> A,B,C
        // Clone -> B,C
        // DebugClone-> B,C
        let bound_oject_pool = intersection_bound_trait_to_object_pool(
            // Set of bound
            ir_type_impl_traits_pool,
            // TA => "TA",Vec<Object>
            &trait_to_impl_pool,
        );

        // opts
        OptArrayBuilder::default()
            .configs(configs)
            .crate_info(crate_info)
            .root_src_file(root_src_file)
            .trait_to_impl_pool(trait_to_impl_pool)
            .bound_oject_pool(bound_oject_pool)
            .build()
            .unwrap()
    }
}

/// api
impl OptArray {
    // fn remove(content: String, keys: Vec<String>) -> String {
    //     content
    //         .split("\n")
    //         .filter(|line| keys.iter().all(|key| !line.contains(key)))
    //         .join("\n")
    // }

    fn to_translation(s: &str) -> String {
        format!("{}_translate", s)
    }
    pub fn run_generate_bound_enum(&self) {
        // generate enum file
        if !self.bound_oject_pool.is_empty() {
            self.generate_impl_file();
            addition_with_path(
                &self.root_src_file,
                vec!["mod bridge_generated_bound;".to_owned()],
            );
        }
    }
    pub fn run_generate_api_translation(&self) {
        self.configs
            .get_translate()
            .iter()
            .clone()
            .for_each(|(s, d)| self.handle_translate(s, d));

        // handle lib.rs
        let ds: Vec<String> = self
            .get_api_paths()
            .iter()
            .map(|s| Self::to_translation(s))
            .map(|d| format!("mod {};", d))
            .collect();

        addition_with_path(&self.root_src_file, ds);
    }
    // fn get_translate_pool()
    fn handle_translate(&self, s: impl AsRef<str>, d: impl AsRef<str>) {
        let source_rust_content = fs::read_to_string(s.as_ref()).unwrap();
        let mut dest_rust_content = self
            .bound_oject_pool
            .keys()
            .sorted_by(|a, b| Ord::cmp(&a.len(), &b.len()))
            .map(|k| (k.join(" + "), k.iter().join("_")))
            .map(|(s, d)| {
                (
                    format!(": impl {}", s),
                    format!(": {}Enum", d.to_case(Case::Pascal)),
                )
            })
            .fold(source_rust_content, |mut state, (s, d)| {
                state = state.replace(&s, &d);
                state
            });
        dest_rust_content += "\npub use crate::bridge_generated_bound::*;";
        fs::write(d.as_ref(), dest_rust_content).unwrap();
    }
    pub fn run_flutter_rust_bridged(&self) {
        let mut configs = self.configs.clone();
        for mut opt in configs.iter_mut() {
            opt.rust_input_path = opt.rust_input_path.replace(".rs", "_translate.rs");
        }
        let all_symbols = get_symbols_if_no_duplicates(&configs).unwrap();
        for config in configs.iter() {
            frb_codegen(config, &all_symbols).unwrap();
        }
    }
}
