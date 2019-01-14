use walkdir::WalkDir;
use std::path::{PathBuf, Path};

/// Convert of a collection of input files and/or directories into an iterator
/// of just excel workbooks (.xls, .xlsm, .xlsx)
pub fn all_workbooks<'a>(inputs: &'a [PathBuf]) -> impl Iterator<Item = PathBuf> + 'a
{
    inputs.iter()
        .flat_map(|entry| { 
            WalkDir::new(&entry)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
        })
        .filter(is_excel)
        .filter(is_not_excel_temp)
}

/// Check the extension of a Path to see if it is an excel workbook
fn is_excel<P: AsRef<Path>>(file: &P) -> bool {
    if let Some(ex) = file.as_ref().extension() {
        match &*ex.to_string_lossy() {
            "xlsx" => true,
            "xls" => true,
            "xlsm" => true,
            _ => false,
        }
    } else { false }
}

/// Check if an excel file is a not temp file
fn is_not_excel_temp<P: AsRef<Path>>(file: &P) -> bool {
    !file.as_ref()
        .file_stem()
        .filter(|s| s.to_string_lossy().starts_with("~"))
        .is_some()
}
