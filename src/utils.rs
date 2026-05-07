use std::path::Path;

pub fn display_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if path_str.starts_with(r"\\?\") {
        path_str[4..].to_string()
    } else {
        path_str.to_string()
    }
}
