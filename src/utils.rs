use std::path::PathBuf;

use walkdir::WalkDir;

pub fn files_with_extension_from_dir(dir: &PathBuf, extension: &str) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.ends_with(extension) && entry.file_type().is_file() {
                result.push(entry.into_path());
            }
        }
    }

    result
}
