use failure::Fail;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// walk image base directory
// limit to files with image extensions (.png, .jpg, .gif, .mp4, .tif)?
// list only images that are not in the metadata file
// image directory needs to be flat for it to be read into db

fn image_exts() -> HashSet<&'static str> {
    ["png", "jpg", "gif", "tif", "mp4"]
        .iter()
        .cloned()
        .collect()
}

pub fn write_image_section_header<W: Write>(mut wtr: W) -> io::Result<()> {
    writeln!(wtr, "\n## Unreferenced Image Check")
}

pub fn check_unref_images<W: Write>(
    ref_imgs: &HashSet<PathBuf>,
    base: &Path,
    mut wtr: W,
) -> Result<(), ImgCheckErr> {
    use ImgCheckErr::*;

    let extensions = image_exts();

    if !base.is_dir() {
        return Err(NotDir(base.to_string_lossy().into()));
    }

    fs::read_dir(base)
        .map_err(|e| AccessDir(base.to_string_lossy().into(), e))
        .and_then(|iter| {
            for entry in iter {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    writeln!(wtr, "* Found subdirectory in image folder")?;
                    writeln!(wtr, "  * Image directory needs to be flat for import")?;
                    writeln!(wtr, "  * {}", path.display())?;

                    continue;
                }

                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy();
                    // maybe record the non image file type?
                    if !extensions.contains(&*ext) {
                        continue;
                    }
                }

                if !ref_imgs.contains(&path) {
                    writeln!(wtr, "* Found unreferenced image: {}", path.display())?;
                }
            }

            Ok(())
        })
}

/// Report any duplicate file stems in the MIFC-Images files
pub fn duplicate_file_stems(
    stem_count: &HashMap<String, u32>,
    mut wtr: impl Write,
) -> io::Result<()> {
    writeln!(wtr, "\n## Repeated File Stem Check")?;

    for (stem, count) in stem_count.iter().filter(|(_, v)| **v > 1) {
        writeln!(wtr, "* Found repeated ({}) file stem: {}", count, stem)?;
    }

    Ok(())
}

#[derive(Debug, Fail)]
pub enum ImgCheckErr {
    #[fail(display = "Image directory path '{}' was not a directory", _0)]
    NotDir(String),
    #[fail(display = "Couldn't open image directory '{}' for checking", _0)]
    AccessDir(String, #[fail(cause)] io::Error),
    #[fail(display = "Io Error when checking for unreferenced images")]
    Io(#[fail(cause)] io::Error),
}

impl From<io::Error> for ImgCheckErr {
    fn from(e: io::Error) -> Self {
        ImgCheckErr::Io(e)
    }
}
