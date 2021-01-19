use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::path::Path;

use reqwest::blocking::Client;
use serde_derive::Deserialize;

#[derive(Debug)]
pub struct VocabMaps {
    pub targets: VocabSet,
    pub methods: VocabSet,
    pub locations: VocabSet,
    pub units: VocabSet,
    pub chips: VocabSet,
}

#[derive(Debug)]
pub struct VocabSet {
    pub values: HashSet<Box<str>>,
    pub case_sensitive: bool,
}

impl VocabMaps {
    pub fn new(chips: &Path) -> Result<Self> {
        macro_rules! MPS_API_BASE {
            () => {
                "https://mps.csb.pitt.edu/api/"
            };
        }
        const MPS_TARGETS: &str = concat!(MPS_API_BASE!(), "targets/");
        const MPS_METHODS: &str = concat!(MPS_API_BASE!(), "methods/");
        const MPS_LOCATIONS: &str = concat!(MPS_API_BASE!(), "locations/");
        const MPS_UNITS: &str = concat!(MPS_API_BASE!(), "units/");

        let client = Client::new();
        let targets = VocabSet::download(MPS_TARGETS, &client, true)?;
        let methods = VocabSet::download(MPS_METHODS, &client, true)?;
        let locations = VocabSet::download(MPS_LOCATIONS, &client, false)?;
        let units = VocabSet::download(MPS_UNITS, &client, true)?;
        let chips = VocabSet::from_csv(chips, "Name", true)?;

        Ok(Self {
            targets,
            methods,
            locations,
            units,
            chips,
        })
    }
}

impl VocabSet {
    fn from_csv(p: &Path, col: &str, case_sensitive: bool) -> Result<Self> {
        let mut rdr =
            csv::Reader::from_path(p).with_context(|| anyhow!("opening CSV {}", p.display()))?;
        let target_col = rdr
            .headers()
            .map(|hdr| hdr.iter().position(|c| c == col))?
            .ok_or_else(|| anyhow!("Missing column {} in CSV {}", col, p.display()))?;

        let mut record = csv::StringRecord::new();
        let mut values = HashSet::new();
        while rdr.read_record(&mut record)? {
            let entry = &record[target_col];
            if entry == "" {
                continue;
            }

            if case_sensitive {
                values.insert(entry.into());
            } else {
                values.insert(entry.to_lowercase().into());
            }
        }

        Ok(Self {
            values,
            case_sensitive,
        })
    }

    pub fn download(url: &str, client: &Client, case_sensitive: bool) -> Result<Self> {
        let info: Vec<ComponentInfo> = client
            .get(url)
            .send()
            .with_context(|| anyhow!("sending request to {}", url))?
            .json()
            .with_context(|| anyhow!("parsing JSON from {}", url))?;

        let values = info
            .into_iter()
            .map(|ComponentInfo { name, .. }| {
                if case_sensitive {
                    name.into_boxed_str()
                } else {
                    name.to_lowercase().into_boxed_str()
                }
            })
            .collect();

        Ok(Self {
            values,
            case_sensitive,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ComponentInfo {
    id: usize,
    name: String,
}
