use super::*;

pub trait OptsUtilTrait: AsRef<[Opts]> {
    fn get_api_paths(&self) -> HashSet<String> {
        self.as_ref()
            .iter()
            .map(|config| {
                config
                    .rust_input_path
                    .split('/')
                    .last()
                    .map(|s| s.split('.').next())
                    .unwrap()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    fn get_translate(&self) -> Vec<(String, String)> {
        self.as_ref()
            .iter()
            .map(|config| &config.rust_input_path)
            .map(|s| (s.to_owned(), s.replace(".rs", "_translate.rs")))
            .collect()
    }
}

impl OptsUtilTrait for [Opts] {}
impl OptsUtilTrait for Vec<Opts> {}
