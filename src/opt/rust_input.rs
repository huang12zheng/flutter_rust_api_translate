/// make [input file]s to a single irfile
/// 1. get irfile
/// 2. collect
mod file;
pub use file::*;
mod func;
use super::*;
pub use func::*;
pub use itertools::Itertools;

pub type IrImplTraitPool = HashSet<IrTypeImplTrait>;

#[derive(Debug, Clone, Default)]

pub struct IrFile {
    // pub funcs: Vec<IrFunc>,
    // pub struct_pool: StructPool,
    // pub enum_pool: EnumPool,
    // pub trait_to_impl_pool: TraitToImplPool,
    pub ir_type_impl_traits: IrImplTraitPool,
    // pub parsed_impl_traits: Vec<IrTypeImplTrait>,
    // pub has_executor: bool,
}

impl IrFile {
    pub fn get_ir_info(rust_input_path: &str) -> IrFile {
        // pub fn get_ir_info(rust_input_path: &str, crate_info: &Crate) -> IrFile {
        let file_ast = get_file(rust_input_path);

        // info!("Phase: Parse AST to IR");
        let ir_type_impl_traits = get_sig_args(&file_ast);

        IrFile {
            ir_type_impl_traits,
        }
    }
}

pub type TraitToImplPool = HashMap<String, Vec<Impl>>;

pub trait RustInputInfo {
    fn get_irs(&self) -> Vec<IrFile>;
    fn collect_irs(&self) -> IrFile;
}

impl RustInputInfo for Vec<Opts> {
    fn get_irs(&self) -> Vec<IrFile> {
        self.iter()
            .map(|config| {
                // let origen_irfile = config.get_ir_file();

                IrFile::get_ir_info(&config.rust_input_path)
            })
            .collect()
    }

    fn collect_irs(&self) -> IrFile {
        let files = self.get_irs();
        let mut file = files
            .into_iter()
            .fold(IrFile::default(), |mut state, event| {
                state.ir_type_impl_traits.extend(event.ir_type_impl_traits);
                state
            });
        file.ir_type_impl_traits = file.ir_type_impl_traits.into_iter().unique().collect();
        file
    }
}
