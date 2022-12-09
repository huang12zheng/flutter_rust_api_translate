use super::*;
pub fn remove_dependencies(configs: &[Opts], root_src_file: impl AsRef<str>) {
    // mutst remove generate source dependencies
    let mut ds: Vec<String> = configs
        .get_api_paths()
        .iter()
        .map(|s| OptArray::to_translation(s))
        .collect();
    ds.push("mod bridge_generated_bound;".to_owned());
    // no need handle api.file use; due to we are copy.
    remove_with_path(root_src_file.as_ref(), ds);

    fs::remove_file(BOUND_PATH).err();

    // handle_translate() call for each api file
    configs.get_translate().iter().for_each(|(_s, d)| {
        fs::remove_file(d).err();
    });
}
