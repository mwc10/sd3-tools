use walkdir::WalkDir;
use std::path::{PathBuf};

#[inline]
pub fn iter_csv_paths<'i>(inputs: &'i [PathBuf]) -> impl Iterator<Item = PathBuf> + 'i {
    inputs.iter()
        .flat_map(|entry| { 
            WalkDir::new(&entry)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
        })
        .filter(is_csv)
}

#[inline]
fn is_csv(file: &PathBuf) -> bool {
    file.extension()
        .map(|ex| ex == "csv")
        .unwrap_or(false) 
}
