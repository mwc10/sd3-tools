use crate::vocab::*;
use calamine::{self, RangeDeserializerBuilder, Reader};
use failure::{format_err as ferr, Error, ResultExt};
use sd3::MifcImage;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn qc_images<W: Write>(metadata: &Path, vocab: VocabMaps, mut output: W) -> Result<(), Error> {
    let mut wb = calamine::open_workbook_auto(metadata)
        .context("opening input image metadata excel file")?;
    let first_sheet = wb
        .sheet_names()
        .get(0)
        .ok_or_else(|| ferr!("No sheets in metadata workbook"))?
        .clone();
    let range = wb.worksheet_range(&first_sheet).unwrap()?;
    let metadata_iter = RangeDeserializerBuilder::new().from_range(&range)?;
    let check_row = make_vocab_checker(&vocab);

    // Start writing the output file
    write_output_prologue(&mut output, &metadata.to_string_lossy())?;
    // Check the controlled vocab and image path for each row in the metadata excel file,
    // while also collecting a list of image file names
    let expected_images: HashSet<PathBuf> = metadata_iter
        .map(Into::into)
        .enumerate()
        .map(|(i, record): (usize, Result<MifcImage, _>)| match record {
            Ok(row) => RowInfo::new(i, row.file.as_str(), check_row(i, &row)),
            Err(e) => RowInfo::new(
                i,
                None,
                format!("* issue parsing line {} in metadata file: {}", i + 2, e),
            ),
        })
        .try_fold(HashSet::new(), |mut acc, info| -> io::Result<_> {
            if let Some(issue) = info.issues {
                writeln!(&mut output, "{}", issue)?;
            }
            if let Some(img) = info.img_name {
                acc.insert(img);
            }

            Ok(acc)
        })?;

    log::info!("{:?}", &expected_images);
    Ok(())
}

struct RowInfo {
    #[allow(dead_code)]
    number: usize,
    img_name: Option<PathBuf>,
    issues: Option<String>,
}

impl RowInfo {
    fn new<'a, N, I>(number: usize, img: N, iss: I) -> Self
    where
        N: Into<Option<&'a str>>,
        I: Into<Option<String>>,
    {
        let img_name = img
            .into()
            .and_then(|p| Path::new(&p).file_name().map(PathBuf::from));

        Self {
            number,
            img_name,
            issues: iss.into(),
        }
    }
}

fn write_output_prologue<W: Write>(mut wtr: W, file: &str) -> io::Result<()> {
    writeln!(wtr, "# Image QC for \"{}\"\n", file)?;
    //TODO: Add run date?
    writeln!(wtr, "## Image Metadata File Vocab and Image Path Checks")
}

fn make_vocab_checker<'m>(
    allowed: &'m VocabMaps,
) -> impl Fn(usize, &MifcImage) -> Option<String> + 'm {
    use std::iter::once;

    let check_target = make_checker("Target/Analyte", &allowed.targets);
    let check_method = make_checker("Method/Kit", &allowed.methods);
    let check_unit = make_checker("Value Unit", &allowed.units);
    let check_location = make_checker("Sample Location", &allowed.locations);
    let check_chip = make_checker("Chip ID", &allowed.chips);

    move |i, row| {
        once(check_target(i, row))
            .chain(once(check_method(i, row)))
            .chain(once(check_unit(i, row)))
            .chain(once(check_location(i, row)))
            .chain(once(check_chip(i, row)))
            .chain(once(check_image(i, row)))
            .filter_map(|x| x)
            .fold(None, |s: Option<String>, iss| {
                s.map(|mut s| {
                    s.push('\n');
                    s.push_str(&iss);
                    s
                })
                .or_else(|| Some(format!("### Row {}\n{}", i + 2, iss)))
            })
    }
}

/// A factory to create functions that check for metadata fields in the MIFC file
fn make_checker<'m>(
    col_name: &'m str,
    allowed_vocab: &'m HashSet<Box<str>>,
) -> impl Fn(usize, &MifcImage) -> Option<String> + 'm {
    move |i, row| {
        let value = row.get_vocab_field(col_name);

        if !allowed_vocab.contains(value) {
            Some(format!(
                "* row {} field \"{}\" is not in MPS: {}",
                i + 2,
                col_name,
                value
            ))
        } else {
            None
        }
    }
}
// A specialized checking function for images that looks if each image path (a) exists and (b) is a file
fn check_image(i: usize, row: &MifcImage) -> Option<String> {
    let img = Path::new(&row.file);
    let find_err = |i, e| {
        format!(
            "* row {} image path error: {}\n  * path: {}",
            i,
            e,
            &img.display()
        )
    };
    let file_err = |i| format!("* row {} image path is not a file: {}", i, &img.display());

    std::fs::metadata(&img)
        .map_err(|e| find_err(i + 2, e))
        .and_then(|f| {
            if f.is_file() {
                Ok(())
            } else {
                Err(file_err(i + 2))
            }
        })
        .err()
}
