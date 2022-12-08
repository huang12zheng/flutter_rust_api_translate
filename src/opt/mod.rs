mod crate_info;
use crate_info::*;
mod customize;
// use customize::*;
mod generate;
use derive_builder::Builder;
// use generate::*;
mod rust_input;
use rust_input::*;

pub(crate) use convert_case::{Case, Casing};
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::fs;
pub(crate) use std::hash::Hash;
pub(crate) use std::iter::FromIterator;
pub(crate) use syn::{punctuated::Punctuated, token::Colon, *};

pub use lib_flutter_rust_bridge_codegen::Opts;

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
    pub fn new(configs: &[Opts]) -> Self {
        // let mut opts = Self::new_without_resolve(configs);
        let crate_info = Crate::new(&configs[0].manifest_path);
        let root_src_file = crate_info.root_src_file.to_str().unwrap().into();
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

impl OptArray {
    // fn remove(content: String, keys: Vec<String>) -> String {
    //     content
    //         .split("\n")
    //         .filter(|line| keys.iter().all(|key| !line.contains(key)))
    //         .join("\n")
    // }
    fn remove_with_path(path: &str, keys: Vec<String>) {
        let content = fs::read_to_string(path).unwrap();
        let content = content
            .split('\n')
            .filter(|line| keys.iter().all(|key| !line.contains(key)))
            .join("\n");
        fs::write(path, content).unwrap();
    }
    fn to_translation(s: &str) -> String {
        format!("{}_translate", s)
    }
    pub fn run_generate_bound_enum(&self) {
        // remove generate source dependencies
        let mut ds: Vec<String> = self
            .get_api_paths()
            .iter()
            .map(|s| Self::to_translation(s))
            .collect();
        ds.push("mod bridge_generated_bound;".to_owned());
        // no need handle api.file use; due to we are copy.
        Self::remove_with_path(&self.root_src_file, ds);

        // generate enum file
        if !self.bound_oject_pool.is_empty() {
            self.generate_impl_file();
        }
    }
    pub fn run_generate_api_translation(&self) {
        let map = self
            .configs
            .iter()
            .map(|config| &config.rust_input_path)
            .map(|s| (s.to_owned(), s.replace(".rs", "_translate.rs")));
        // handle_translate() call for each api file
        map.clone().for_each(|(s, d)| self.handle_translate(s, d));

        // handle lib.rs
        let root_rust_content = fs::read_to_string(&self.root_src_file).unwrap();
        let mut ds: Vec<String> = self
            .get_api_paths()
            .iter()
            .map(|s| Self::to_translation(s))
            .map(|d| format!("mod {};", d))
            .collect();
        ds.push("mod bridge_generated_bound;".to_owned());
        let addition_content = ds.join("\n");
        fs::write(
            &self.root_src_file,
            root_rust_content + "\n" + &addition_content,
        )
        .unwrap();
    }
    // fn get_translate_pool()
    fn handle_translate(&self, s: String, d: String) {
        let source_rust_content = fs::read_to_string(s).unwrap();
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
        dest_rust_content += "\npub use crate::bridge_generated_bound::*;\n";
        fs::write(d, dest_rust_content).unwrap();
    }
}
