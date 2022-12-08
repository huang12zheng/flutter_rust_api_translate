use super::*;

impl OptArray {
    pub(crate) fn remove_gen_mod(&self, file_path: impl Display) {
        Command::new("sh")
            .args([
                "-c",
                format!("sed -i '' '/.*mod .*bridge_generated.*/d' {file_path}").as_str(),
            ])
            .spawn()
            .ok();
    }
    pub(crate) fn remove_gen_use(&self, file_path: impl Display) {
        Command::new("sh")
            .args([
                "-c",
                format!("sed -i '' '/.*use .*bridge_generated_bound.*/d' {file_path}").as_str(),
            ])
            .spawn()
            .ok();
    }
    pub(crate) fn gen_mod(&self, file_path: impl Display) {
        Command::new("sh")
            .args([
                "-c",
                format!("echo 'mod bridge_generated_bound;' >> {file_path}").as_str(),
            ])
            .spawn()
            .ok();
    }

    pub(crate) fn gen_use(&self, file_path: impl Display) {
        Command::new("sh")
            .args([
                "-c",
                format!("echo 'pub use crate::bridge_generated_bound::*;' >> {file_path}").as_str(),
            ])
            .spawn()
            .ok();
    }
}
