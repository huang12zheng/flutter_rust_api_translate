use super::*;

impl OptArray {
    pub fn run_generate_dart_ffi_file(&self) {
        self.generate_dart_ffi_file();
        if self.configs[0].wasm_enabled {
            self.generate_dart_wasm_file()
        }
    }
}
impl OptArray {
    pub fn generate_dart_ffi_file(&self) {
        let mut lines = String::new();
        lines += r"
// Autogen: This file initializes the dynamic library and connects it
import 'dart:ffi';
import 'dart:io' as io;
";
        lines += self
            .configs
            .iter()
            .map(|opt| opt.dart_output_path.split('/').last().unwrap())
            .map(|path| format!("import '{path}';\nexport '{path}';"))
            .join("\n")
            .as_str();
        lines += "final _dylib = io.Platform.isWindows ? 'native.dll' : 'libnative.so';";
        lines += self
            .configs
            .iter()
            .map(|opt| opt.class_name.as_str())
            .map(gen_ffi_code)
            .join("\n")
            .as_str();
        fs::write(format!("{FFI_PATH}.dart"), lines).unwrap();
    }
    pub fn generate_dart_wasm_file(&self) {
        let mut lines = String::new();
        lines += r"
// Autogen: This file initializes the dynamic library and connects it
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
";
        lines += self
            .configs
            .iter()
            .map(|opt| opt.dart_output_path.split('/').last().unwrap())
            .map(|path| format!("import '{path}';\nexport '{path}';"))
            .join("\n")
            .as_str();

        lines += self
            .configs
            .iter()
            .map(|opt| opt.class_name.as_str())
            .map(gen_wasm_code)
            .join("\n")
            .as_str();
        fs::write(format!("{FFI_PATH}_web.dart"), lines).unwrap();
    }
}

fn gen_ffi_code(ident: &str) -> String {
    let snake = ident.to_case(Case::Snake);
    // final _{snake}_dylib = io.Platform.isWindows ? '{snake}.dll' : 'lib{snake}.so';
    format!(
        r"
final {ident} api_{snake} = {ident}Impl(io.Platform.isIOS || io.Platform.isMacOS
    ? DynamicLibrary.executable()
    : DynamicLibrary.open(_dylib));
"
    )
}

fn gen_wasm_code(ident: &str) -> String {
    let snake = ident.to_case(Case::Snake);
    format!(
        r"

final {ident} api{snake} = {ident}Impl.wasm(
    WasmModule.initialize(kind: const Modules.noModules(root: 'pkg/native')),
);
"
    )
}
