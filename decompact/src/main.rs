mod input;

use crate::input::*;
use failure::Error;
use flexi_logger::{default_format, Logger};
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Decompact a "compacted" file of chip+time vs readout values into
/// the standard MIF-C format
struct Opts {
    #[structopt(parse(from_os_str))]
    /// Compacted input CSV file
    input: PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    /// Target/Method config toml file
    config: PathBuf,
    #[structopt(parse(from_os_str))]
    /// Output CSV file location, or stdout if not present
    output: Option<PathBuf>,
    /// Print debug info based on the number of "v"s passed
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: usize,
}

fn main() {
    let opts = Opts::from_args();
    let log_level = match opts.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    Logger::with_str(log_level)
        .format(default_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    if let Err(e) = run(opts) {
        errlog::print_chain(&e);
        ::std::process::exit(1);
    }
}

fn run(opts: Opts) -> Result<(), Error> {
    info!("{:#?}", &opts);

    let mut rdr = csv::Reader::from_path(&opts.input)?;
    let headers = rdr.headers()?;

    let header_lut = dbg!(parse_header(&headers))?;

    Ok(())
}
