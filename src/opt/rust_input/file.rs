use super::*;
pub fn get_file(rust_input_path: &str) -> File {
    // info!("Phase: Parse source code to AST");
    let source_rust_content = fs::read_to_string(rust_input_path)
        .unwrap_or_else(|_| panic!("panic with file: {}", &rust_input_path));
    // AST
    syn::parse_file(&source_rust_content).unwrap()
}
