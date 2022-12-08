mod crate_info;
use crate_info::*;
mod customize;
// use customize::*;
mod generate;
use derive_builder::Builder;
// use generate::*;
mod rust_input;
use rust_input::*;

pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::fmt::Display;
pub(crate) use std::fs;
pub(crate) use std::hash::Hash;
pub(crate) use std::iter::FromIterator;
pub(crate) use std::process::Command;
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
    // pub fn new_without_resolve(configs: &[Opts]) -> Self {
    //     OptArray {
    //         configs: configs.to_owned(),
    //         // irs: Vec::new(),
    //         trait_to_impl_pool: HashMap::new(),
    //         // parsed_impl_traits: HashSet::new(),
    //         bound_oject_pool: HashMap::new(),
    //         root_src_file: String::new(),
    //         crate_info: None,
    //     }
    // }

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

fn intersection_bound_trait_to_object_pool(
    ir_type_impl_traits_pool: HashSet<IrTypeImplTrait>,
    trait_to_impl_pool: &HashMap<String, Vec<Impl>>,
) -> HashMap<Vec<String>, HashSet<String>> {
    ir_type_impl_traits_pool
        .iter()
        .flat_map(|ty| &ty.trait_bounds)
        .for_each(|trait_| {
            if !trait_to_impl_pool.contains_key(trait_) {
                panic!("loss impl {} for some self_ty", trait_);
            }
        });
    // ir_type_impl_traits_pool.iter().for_each(|type_impl_trait| {
    //     type_impl_trait.trait_bounds.iter().for_each(|trait_| {
    //         // Check whether the trait bound is capable of being used
    //         // ~~return None if param unoffical~~
    //         if !trait_to_impl_pool.contains_key(trait_) {
    //             panic!("loss impl {} for some self_ty", trait_);
    //         }
    //     });
    // });
    ir_type_impl_traits_pool
        .into_iter()
        .map(|ty| ty.trait_bounds)
        .map(|trait_bounds| {
            let sets = trait_bounds.iter().map(|trait_| {
                let impls = trait_to_impl_pool.get(trait_).unwrap();
                let iter = impls.iter().map(|impl_| impl_.self_ty.to_string());
                HashSet::from_iter(iter)
            });

            let mut iter = sets;

            let intersection_set = iter
                .next()
                .map(|set: HashSet<String>| iter.fold(set, |set1, set2| &set1 & &set2))
                .unwrap();
            (trait_bounds, intersection_set)
        })
        .collect()
}

impl OptArray {
    pub fn run_generate_bound_enum(&self) {
        // remove generate source dependencies
        self.remove_gen_mod(&self.root_src_file);
        for config in self.configs.iter() {
            let api_file = config.rust_input_path.clone();
            self.remove_gen_use(api_file);
        }

        if !self.bound_oject_pool.is_empty() {
            self.generate_impl_file();

            // generate source dependencies
            self.gen_mod(&self.root_src_file);
            for config in self.configs.iter() {
                let api_file = config.rust_input_path.clone();
                self.gen_use(api_file);
            }
        }
    }
}

mod handle_use_info;
