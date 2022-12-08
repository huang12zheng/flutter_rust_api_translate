## Why
Because I can't figure out a good way to get into flutter_rust_bridge. I decided to open a new build for myself

## example
```rs
use flutter_rust_api_translate::*;
// use lib_flutter_rust_bridge_codegen::{
//     config_parse, frb_codegen, get_symbols_if_no_duplicates, RawOpts,
// };

/// Path of input Rust code
const RUST_INPUT: &str = "src/api.rs";
/// Path of output generated Dart code
const DART_OUTPUT: &str = "../dart/lib/bridge_generated.dart";

fn main() {
    // Tell Cargo that if the input Rust code changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", RUST_INPUT);
    // Options for frb_codegen
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
    // get opts from raw opts
    let configs = config_parse(raw_opts);

    let opts = OptArray::new(&configs);
    opts.run_generate_bound_enum();
    opts.run_generate_api_translation();
    opts.run_flutter_rust_bridged();
}
```

```sh
# ls flutter_rust_api_translate/test/flutter_rust_bridge/frb_example/pure_dart/rust/src
api.rs                    bridge_generated.web.rs   new_module_system
api_translate.rs          bridge_generated_bound.rs new_module_system.rs
bridge_generated.io.rs    data.rs                   old_module_system
bridge_generated.rs       lib.rs
```

```rs
// lib.rs
mod bridge_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
/// impl_trait: |Debug|fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
/// impl_trait: |Default|
/// handle_opaque: HideData|NonSendHideData
/// add_box: ExoticOptionals
mod api;
mod data;
mod new_module_system;
mod old_module_system;

mod api_translate;
mod bridge_generated_bound;
```

## Type Impl Trait

In Rust, we can use signature inputs or signature output as func args.
For example, `pub fn tt(t: impl Serialize) {}` is work.
And `Enum` would transform between dart and rust. So we can translate `TypeImplTrait` to `Enum` make something very imaginative.

## some code that writing by hand can work

  + part 1
  ```rs
  pub enum SerializeEnum {
      Record(Record),
      SerEnum(SerEnum),
  }

  impl Serialize for SerializeEnum {
      fn serialize<S>(&self, __serializer: S) -> Result<S::Ok, S::Error>
      where
          S: Serializer,
      {
          match *self {
              SerializeEnum::Record(ref __field0) => __field0.serialize(__serializer),
              SerializeEnum::SerEnum(ref __field0) => __field0.serialize(__serializer),
          }
      }
  }
  ```
  > Some inspiration comes from enum_dispatch, Thanks to the library
and translate signature to 
  + part 2
  ```rs
  pub fn tt(t: SerializeEnum) {}
  ```


## automatically generated results with build.rs
```rs
fn wire_tt_impl(port_: MessagePort, t: impl Wire2Api<SerializeEnum> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "tt",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_t = t.wire2api();
            move |task_callback| Ok(tt(api_t))
        },
    )
}
```
The `Enum of part1 above` is generated automatically, and you don't need to modify your original function in `part 2`, because wire has already implemented translation from `Wire2Api<Impl Serialize>` to `Wire2Api<SerializeEnum>`.

[If you look at the details of the generated code (rust and dart)](https://github.com/huang12zheng/flutter_rust_bridge/blob/51d2d7bedc6a99493bb8b77a84d0d4a82488e650/frb_codegen/src/ir/file.rs),
it translates the IrImplTrait into EnumRef when check dependent function parameters. And then directly calls the TypeEnumRefGenerator to generate the code.


## Some information for reference:
* a more doc about IrImplTrait
```
'impl Aa+Cc' make results to the 'AaCcEnum'
```

- [discuss 866](https://github.com/fzyzcjy/flutter_rust_bridge/discussions/866)

* If you have a function argument as the Type Impl Trait, do not manually implement enum with the same name as the translation. This could conflict.

* some problem (like large_enum_variant and private) maybe need to handle by yourself.