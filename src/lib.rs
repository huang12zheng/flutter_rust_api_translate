pub use lib_flutter_rust_bridge_codegen::{
    config_parse, frb_codegen, get_symbols_if_no_duplicates, RawOpts,
};
mod opt;
pub use opt::*;
// pub use generate_template::*;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Path of input Rust code
    pub const RUST_INPUT: &str = "src/api.rs";
    /// Path of output generated Dart code
    pub const DART_OUTPUT: &str = "../dart/lib/bridge_generated.dart";

    #[test]
    fn it_works() {
        let raw_opts = RawOpts {
            // Path of input Rust code
            rust_input: vec![RUST_INPUT.to_string()],
            // Path of output generated Dart code
            dart_output: vec![DART_OUTPUT.to_string()],
            wasm: true,
            dart_decl_output: Some("../dart/lib/bridge_definitions.dart".into()),
            dart_format_line_length: 120,
            // for other options use defaults
            ..Default::default()
        };
        let configs = config_parse(raw_opts);
        let opts = OptArray::new(&configs);
        opts.run_generate_bound_enum();
    }
}
