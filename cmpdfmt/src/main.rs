mod proc_inputs;
mod convert;
mod output;

use structopt::{StructOpt};
use failure::{Error, ResultExt};
use log::{debug};
use flexi_logger::{Logger, default_format};
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Any number of input compound columnar csv files or directories containing those csv files
    #[structopt(name = "INPUT", parse(from_os_str))]
    input: Vec<PathBuf>,
    /// Append to input filename for output filename; defaults to "mifc"
    #[structopt(short = "a", long = "append")]
    append: Option<String>,
    /// If present, directory in which output files are created
    #[structopt(short = "o", long = "out-dir", parse(from_os_str))]
    out_dir: Option<PathBuf>, 
    /// Other, special propagating terms besides stock and reservoir
    #[structopt(short = "t", long = "term", number_of_values = 1)]
    other_terms: Vec<String>,
    /// Output the conversion of each file to stdout instead of writing to files
    #[structopt(long = "stdout")]
    stdout: bool,
    /// Set the verbosity level (1, 2, or 3)
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8,
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
        errlog::print_chain(&e);
        ::std::process::exit(1);
    }
}

fn run(opts: Opt) -> Result<(), Error> {
    let inputs = &opts.input;
    debug!("Input files: {:?}", inputs);
    let csv_paths = proc_inputs::iter_csv_paths(inputs);
    convert::cmpd_csv_to_mifc(csv_paths, &opts)
        .context("couldn't convert inputs")?;

    Ok(())
}
