use super::*;
pub(crate) fn remove_with_path(path: &str, keys: Vec<String>) {
    let content = fs::read_to_string(path).unwrap();
    let content = content
        .split('\n')
        .filter(|line| keys.iter().all(|key| !line.contains(key)))
        .join("\n");
    fs::write(path, content).unwrap();
}
pub(crate) fn addition_with_path(path: impl AsRef<str>, additions: Vec<String>) {
    let content = fs::read_to_string(path.as_ref()).unwrap();
    let split = if content.ends_with('\n') { "" } else { "\n" };
    let content = content + split + &additions.join("\n");
    fs::write(path.as_ref(), content).unwrap();
}
