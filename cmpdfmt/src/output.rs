use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use log::{debug};
use failure::{Error, ResultExt, format_err, bail};

pub fn get_output_wtr(stdout: bool, dir: &Option<&Path>, name: &Path, append: &str) -> Result<Box<dyn Write>, Error> {
    Ok(
        if stdout {
            Box::new(io::stdout()) as Box<dyn Write> 
        } else {
            let output = generate_output_filename(dir, name, append)?;
            debug!("generated output: {:?}", &output);
            let wtr = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)
                .context("opening output file")?;
            
            Box::new(wtr) as Box<dyn Write>
        }
    )
}

fn generate_output_filename(dir: &Option<&Path>, name: &Path, append: &str) -> Result<PathBuf, Error> 
{
    let filestem = name.file_stem().ok_or_else(|| format_err!("input was not a file"))?;
    let parents = name.parent();
    let mut output = dir.map(Path::to_path_buf).unwrap_or_else(|| PathBuf::new());
    if let Some(in_dirs) = parents { output.push(in_dirs); }

    if !output.exists() {
        fs::create_dir_all(&output).context("creating directories for output")?;
    } else if !output.is_dir() {
        bail!("Path <{:?}> passed to \"--out-dir\" is not a directory", &output);
    }

    if append != "" {
        let mut filestem = filestem.to_os_string();
        filestem.push("-");
        filestem.push(&append);
        output.push(filestem);
    } else {
        output.push(filestem);
    }

    output.set_extension("csv");

    Ok(output)
}