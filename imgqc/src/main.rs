mod img;
mod qc;
mod vocab;

use anyhow::{bail, Context, Result};
use flexi_logger::{default_format, Logger};
use log::*;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate::vocab::*;

#[derive(Debug, StructOpt)]
struct Opts {
    /// Path to the MIFC-I metadata excel file (adding CSV later)
    #[structopt(parse(from_os_str))]
    mifc: PathBuf,
    /// File to log QC to, or stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
    /// Directory of image files to check
    #[structopt(short, long, parse(from_os_str))]
    images: Option<PathBuf>,
    /// Study chip and well info file
    #[structopt(short, long, parse(from_os_str))]
    chips: PathBuf,
    /// Debugging Info
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
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
        eprintln!("{:?}", e);
        ::std::process::exit(1);
    }
}

fn run(opts: Opts) -> Result<()> {
    let vocab = VocabMaps::new(&opts.chips).context("creating MPS vocab maps")?;
    // vocab.log()
    let output = create_file_or_stdout(opts.output.as_ref()).context("opening output")?;
    let default_imgdir;
    let user_imgdir = opts.images.as_deref();
    let imgdir = if user_imgdir.is_some() {
        user_imgdir.unwrap()
    } else {
        default_imgdir =
            std::env::current_dir().context("getting cwd for default image directory")?;
        &default_imgdir
    };

    if !imgdir.is_dir() {
        bail!(
            "Image directory path is not a directory: {}",
            imgdir.display()
        );
    }

    debug!("{:#?}\n{:#?}\nimages: {}", &opts, &vocab, imgdir.display());

    qc::qc_images(&opts.mifc, vocab, imgdir, output).context("running image qc")
}

fn create_file_or_stdout<P>(path: Option<P>) -> io::Result<Box<dyn Write>>
where
    P: AsRef<Path>,
{
    use std::fs::File;

    path.map(|p| File::create(&p.as_ref()).map(|f| Box::new(BufWriter::new(f)) as Box<dyn Write>))
        .unwrap_or_else(|| Ok(Box::new(io::stdout()) as Box<dyn Write>))
}
