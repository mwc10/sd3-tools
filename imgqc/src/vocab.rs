use failure::Fail;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct VocabMapsBuilder {
    targets: Option<PathBuf>,
    methods: Option<PathBuf>,
    locations: Option<PathBuf>,
    units: Option<PathBuf>,
    chips: Option<PathBuf>,
}

impl VocabMapsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Set up default paths to vocab CSV files
    pub fn directory_defaults<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        let dir = dir.as_ref();

        if self.targets.is_none() {
            self.targets = {
                let mut targets = dir.to_path_buf();
                targets.push("MPS Database Targets.csv");
                targets.into()
            };
        }

        if self.methods.is_none() {
            self.methods = {
                let mut methods = dir.to_path_buf();
                methods.push("MPS Database Methods.csv");
                methods.into()
            };
        }

        if self.locations.is_none() {
            self.locations = {
                let mut locations = dir.to_path_buf();
                locations.push("MPS Database Locations.csv");
                locations.into()
            };
        }

        if self.units.is_none() {
            self.units = {
                let mut units = dir.to_path_buf();
                units.push("MPS Database Units.csv");
                units.into()
            };
        }

        if self.chips.is_none() {
            self.chips = {
                let mut chips = dir.to_path_buf();
                chips.push("MPS Database Chips.csv");
                chips.into()
            };
        }

        self
    }
    pub fn set_chips<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.chips = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn read_maps(&self) -> Result<VocabMaps, VocabError> {
        // check for nones
        VocabMaps::from_builder(self)
    }
}

#[derive(Debug)]
pub struct VocabMaps {
    pub targets: HashSet<Box<str>>,
    pub methods: HashSet<Box<str>>,
    pub locations: HashSet<Box<str>>,
    pub units: HashSet<Box<str>>,
    pub chips: HashSet<Box<str>>,
}

impl VocabMaps {
    fn from_builder(info: &VocabMapsBuilder) -> Result<Self, VocabError> {
        Ok(Self {
            targets: create_hashset(info.targets.as_ref().unwrap(), "Target")?,
            methods: create_hashset(info.methods.as_ref().unwrap(), "Method")?,
            locations: create_hashset(info.locations.as_ref().unwrap(), "Location")?,
            units: create_hashset(info.units.as_ref().unwrap(), "Unit")?,
            chips: create_hashset(info.chips.as_ref().unwrap(), "Name")?,
        })
    }
}

fn create_hashset(p: &Path, col: &str) -> Result<HashSet<Box<str>>, VocabError> {
    use VocabError::*;

    let mut rdr = csv::Reader::from_path(p).map_err(|e| OpeningVocab(pstr(p), e))?;
    let target_col = rdr
        .headers()
        .map(|hdr| hdr.iter().position(|c| c == col))?
        .ok_or_else(|| VocabError::MissingColumn(col.into(), pstr(p)))?;

    let mut record = csv::StringRecord::new();
    let mut output = HashSet::new();
    while rdr.read_record(&mut record)? {
        if &record[target_col] != "" {
            output.insert(record[target_col].into());
        }
    }

    Ok(output)
}

fn pstr<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().to_string_lossy().into_owned()
}

#[derive(Debug, Fail)]
pub enum VocabError {
    #[fail(display = "couldn't open '{}' for reading", _0)]
    OpeningVocab(String, #[fail(cause)] csv::Error),
    #[fail(display = "required vocab column '{}' not present in '{}'", _0, _1)]
    MissingColumn(String, String),
    #[fail(display = "vocab processing csv error")]
    Csv(#[fail(cause)] csv::Error),
}

impl From<csv::Error> for VocabError {
    fn from(e: csv::Error) -> Self {
        VocabError::Csv(e)
    }
}
