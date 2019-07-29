mod qc;
mod vocab;
mod img;

use failure::{format_err as ferr, Error};
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
    index: PathBuf,
    /// File to log QC to, or stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
    /// Directory of image files to check
    #[structopt(short, long, parse(from_os_str))]
    images: Option<PathBuf>,
    /// Directory containing MPS vocab files
    #[structopt(short, long, parse(from_os_str))]
    vocab: PathBuf,
    /// Study chip info file, or looks for "MPS Chips" in vocab directory
    #[structopt(short, long, parse(from_os_str))]
    chips: Option<PathBuf>,
    /// Debugging Info
    #[structopt(short = "w", long, parse(from_occurrences))]
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
        errlog::print_chain(&e);
        ::std::process::exit(1);
    }
}

fn run(opts: Opts) -> Result<(), Error> {
    let vocab = create_vocab_maps(&opts)?;
    let output = create_file_or_stdout(opts.output.as_ref())?;
    let default_imgdir = std::env::current_dir()?;
    let imgdir = opts
        .images
        .as_ref()
        .map(PathBuf::as_path)
        .or_else(|| Some(default_imgdir.as_path()))
        .filter(|p| p.is_dir())
        .ok_or_else(|| ferr!("Image directory ('-i') path is not a directory"))?;

    debug!("{:#?}\n{:#?}\nimages: {}", &opts, &vocab, imgdir.display());

    qc::qc_images(&opts.index, vocab, imgdir, output)
}

fn create_vocab_maps(opts: &Opts) -> Result<VocabMaps, VocabError> {
    let mut builder = VocabMapsBuilder::new();
    if let Some(p) = opts.chips.as_ref() {
        builder.set_chips(p);
    }

    builder.directory_defaults(&opts.vocab).read_maps()
}

fn create_file_or_stdout<P>(path: Option<P>) -> io::Result<Box<dyn Write>>
where
    P: AsRef<Path>,
{
    use std::fs::File;

    path.map(|p| File::create(&p.as_ref()).map(|f| Box::new(BufWriter::new(f)) as Box<dyn Write>))
        .unwrap_or_else(|| Ok(Box::new(io::stdout()) as Box<dyn Write>))
}
