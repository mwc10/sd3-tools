use failure::{Error, ResultExt, bail, format_err};
use structopt::StructOpt;
use log::{error, warn, info, debug};
use flexi_logger::{Logger, default_format};
use calamine::{Reader, RangeDeserializerBuilder, open_workbook_auto};
use walkdir::WalkDir;

use std::path::{Path, PathBuf};
use std::fmt;
use std::fs::{OpenOptions, self};
use std::ffi::{OsStr};

use sd3::MifcNorm;



#[derive(StructOpt, Debug)]
/// Read an MIFC + normalization info excel workbook and create one normalized MIFC CSV for each sheet
#[structopt(name = "sd3norm", version = "1.1.0")]
struct Opt {
    /// Any number of input mifc+normalization-formatted excel files or directories containing excel files
    #[structopt(name = "INPUT", parse(from_os_str))]
    input: Vec<PathBuf>,
    /// Append to INPUT for output, defaults to "normalized"
    #[structopt(short = "a", long = "append")]
    append: Option<String>,
    /// Print debug info based on the number of "v"s passed
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: usize,
    /// Directory to create output file(s) in
    #[structopt(short = "d", long = "out-dir", parse(from_os_str))]
    out_dir: Option<PathBuf>, 
}


fn main() {
    let opts = Opt::from_args();
    let log_level = match opts.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    Logger::with_str(log_level)
        .format(default_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}",e) );
    
    if let Err(e) = run(opts) {
        error!(": {}", &e);
        for cause in e.iter_causes() {
            error!("caused by: {}", cause);
        }
        match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
            Ok("1") => error!("Backtrace:\n{}", e.backtrace()),
            _ => (),
        }
        ::std::process::exit(1);
    }
}

fn run(opts: Opt) -> Result<(), Error> {
    let inputs = opts.input;    /* A possible mixed collection of directories and file paths */
    let output_directory = opts.out_dir.as_ref().map(PathBuf::as_path);
    /* Get the value to append to the end of the output, or use the default */
    let append_str = opts.append.as_ref().map_or("normalized", String::as_ref);
    
    /* Get output base path by appending the value of optional directory flag */
    debug!("Workbook(s) Input: {:#?}", &inputs);
    debug!("Output directory: {:?}", output_directory);
    debug!("output append: {}", &append_str);

    /* Convert collection of input files and/or directories into a workbook path iterator */
    let workbooks = inputs
        .iter()
        .flat_map(|entry| { 
            WalkDir::new(&entry)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
        })
        .filter(is_excel)
        .filter(is_not_excel_temp)
        .map(|wb| {
            let out = generate_output_base(&wb, output_directory);
            (wb, out, &append_str)
        });
    // TODO: Use a parallel iterator? 
    for (wb, out, app) in workbooks {
        match out {
            Ok(out) =>
                match normalize_workbook(&wb, &out, &app) {
                    Ok(_) => (),
                    Err(e) => {
                        warn!("Couldn't normalize workbook <{}> due to:\n{}", wb.display(), e);
                        continue;
                    }
                }
            Err(e) => {
                warn!("Couldn't generate an output for workbook <{}> due to:\n{}", wb.display(), e);
                continue;
            }
        }
    }

    Ok(())
}

fn normalize_workbook<P, O>(wb_path: P, output_base: O, append: &str) -> Result<(), Error>
where P: AsRef<Path> + fmt::Debug,
      O: AsRef<Path> + fmt::Debug
{
    let mut workbook = open_workbook_auto(&wb_path)
        .context(format!("opening excel workbook <{:?}>", &wb_path))?;
    /* Iterate over the sheets in a workbook */
    let sheets = workbook.sheet_names().to_vec();
    let sheet_sum = sheets.len();

    for (i, s) in sheets.iter().enumerate() {
        let sheet = workbook.worksheet_range(s).unwrap()?;

        /* Generate a writer to output the normalized values from this sheet 
         * If there is only one sheet, don't append the sheet name to the output file name
        **/
        let output = {
            let mut out = output_base.as_ref().to_path_buf();
            let add_sheet = sheet_sum > 1;
            let appended_info = format!("{s_h}{s}-{a}", 
                s_h = if add_sheet {"-"} else {""},
                s = if add_sheet {s} else {""},
                a =  append
            );
            append_file_name(&mut out, &appended_info);
            out
        };

        info!("{:?} - {} (#{}):\nOutput file: {:?}", &wb_path, s, i, &output);

        let mut wtr = csv::Writer::from_writer(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)?
        );

        /* Deserialize the data into SD3 struct, then normalize each possible row, and serialize into output*/
        let rows = match RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&sheet)
        {
            Ok(r) => r,
            Err(e) => {
                warn!("issue parsing sheet <{}> into MIFC normalization format\n{}", s, e);
                fs::remove_file(&output)?;
                continue;
            } 
        };

        for (i, result) in rows.enumerate() {
            let record: MifcNorm = match result {
                Ok(r) => r,
                Err(e) => {
                    info!("couldn't deserializing row {} in {}:\n{}", i+2, s, e); 
                    continue;
                },
            };

            let normalized_row = match record.into_normalized() {
                Ok(n) => n,
                Err(e) => {
                    info!("did not normalize row {} in {}:\n{}", i+2, s, e);
                    continue;
                },
            };

            wtr.serialize(normalized_row)?;
        }
    }
    Ok(())
}

fn append_file_name<S: AsRef<OsStr>>(path: &mut PathBuf, append: S) {
    if path.file_name().is_some() {
        let appended = { 
            let mut new = path.file_stem().unwrap().to_os_string();
            new.push(append);
            if let Some(e) = path.extension() {
                new.push("."); new.push(e);
            }
            new
        };
        path.set_file_name(appended);
    } else {
        path.set_file_name(append);
    }
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

/// Turn the input path and the optional directory argument into an output path buffer
fn generate_output_base(input: &Path, dir: Option<&Path>) -> Result<PathBuf, Error> {
    if let Some(dir) = dir {
        // Get parts of input Path to see if there is a directory structure 
        // to attach to the output "base directory"
        let input_filename = input.file_name()
            .ok_or(format_err!("the input was not a file"))?;
        let input_parent = input.parent();

        // Generate output directory structure, if needed
        let mut output = dir.to_path_buf();
        if let Some(parent_path) = input_parent {
            output.push(parent_path);
        }

        if !output.exists() { 
            fs::create_dir_all(&output).context("making output subdirectory(ies)")?;
        } else if !output.is_dir() { 
            bail!("Path <{:?}> passed to \"--out-dir\" is not a directory", &output);
        }

        output.push(input_filename);
        output.set_extension("csv");

        Ok(output)
    } else {
        Ok(input.with_extension("csv"))
    }
}
