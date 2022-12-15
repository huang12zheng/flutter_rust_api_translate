use super::*;

impl OptArray {
    pub fn generate_impl_file(&self) {
        let (trait_sig_pool, opaque_set, add_box_set) = self.get_sig_from_doc();

        let explicit_api_path = self.get_api_paths();
        let bound_oject_pool = &self.bound_oject_pool;

        let mut lines = String::new();
        // lines += &format!("/* {:?}\n*/\n", bound_oject_pool,);
        // lines += &format!("/* {:?}\n*/\n", self.trait_to_impl_pool,);
        for super_ in explicit_api_path.iter() {
            lines += format!("use crate::{super_}::*;\n").as_str();
        }
        if !opaque_set.is_empty() {
            lines += "use flutter_rust_bridge::RustOpaque;\n";
        }
        for (_, call_fn) in trait_sig_pool.iter() {
            let impl_dependencies = call_fn.impl_dependencies.clone();
            lines += format!("{}\n", impl_dependencies).as_str();
        }
        for (k, v) in bound_oject_pool.iter() {
            lines += format!("pub enum {}Enum {{\n", k.join("")).as_str();
            for struct_ in v.iter() {
                lines += format!(
                    "    {}({}),\n",
                    struct_,
                    if opaque_set.contains(struct_) {
                        format!("RustOpaque<{}>", struct_)
                    } else if add_box_set.contains(struct_) {
                        format!("Box<{}>", struct_)
                    } else {
                        struct_.to_owned()
                    }
                )
                .as_str();
            }
            lines += "}\n".to_string().as_str();
        }

        for (k, v) in bound_oject_pool.iter() {
            let enum_ = format!("{}Enum", k.join(""));
            for trait_ in k.iter() {
                lines += format!("impl {trait_} for {enum_} {{\n").as_str();
                let call_fn = trait_sig_pool
                    .get(trait_)
                    .unwrap_or_else(|| panic!("Error: {:?} with {:?}", trait_sig_pool, trait_));

                for idx in 0..call_fn.sig.len() {
                    lines += format!("{}{{\n", call_fn.sig[idx]).as_str();
                    lines += "match *self {\n".to_string().as_str();
                    for sub_enum in v.iter() {
                        lines += format!(
                            "{enum_}::{sub_enum}(ref __field0) => __field0.{}({}),\n",
                            call_fn.fn_name[idx], call_fn.args[idx]
                        )
                        .as_str();
                    }
                    lines += "}\n".to_string().as_str();
                    lines += "}\n".to_string().as_str();
                }
                lines += "}\n".to_string().as_str();
            }
        }

        fs::write(BOUND_PATH, lines).unwrap();
    }

    pub fn get_api_paths(&self) -> HashSet<String> {
        self.configs.get_api_paths()
    }
}
