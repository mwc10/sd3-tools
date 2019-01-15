use serde_derive::{Deserialize, Serialize};
use failure::{Fail};
use units::{SIError};
use crate::mifc::Mifc;
//use units::SIUnit;

#[derive(Debug, Fail)]
pub enum CmpdDitError {
    #[fail(display = "Couldn't convert <{}> to day, hour, minute time", _0)]
    TimeCvrt(String),
    #[fail(display = "Didn't recognize unit")]
    UnitCvrt(#[cause] SIError)
}

impl From<SIError> for CmpdDitError {
    fn from(e: SIError) -> Self {
        CmpdDitError::UnitCvrt(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmpdDit {
    #[serde(rename = "Group Indicator")]
    group: String,
    #[serde(rename = "Chip ID")]
    id: String, 
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "Method/Kit")]
    method: String,
    #[serde(rename = "Target/Analyte")]
    target: String,
    #[serde(rename = "Result")]
    value: Option<f64>,
    #[serde(rename = "Result Unit")] // TODO: SIUnit?
    value_unit: Option<String>,
    #[serde(rename = "Dilution")]
    dilution: Option<f64>,
    #[serde(rename = "Location")]
    location: String,
    #[serde(rename = "Note (optional)")]
    note: Option<String>,
    #[serde(rename = "Flag (optional)")]
    flag: Option<String>,
    #[serde(rename = "Replicate (optional)")]
    replicate: Option<u32>,
    #[serde(rename = "TCTCxRef (optional)")]
    xref: Option<String>,
    #[serde(rename = "Cell Count")]
    cell_count: Option<f64>,
    #[serde(rename = "Sample Duration")]
    duration: Option<String>,
    #[serde(rename = "Sample Volume (µL)")]
    vol: Option<String>,
}

impl CmpdDit {
    pub fn group(&self) -> &String {
        &self.group
    }

    pub fn chip_id(&self) -> &String {
        &self.id
    }
    pub fn into_mifc(self) -> Result<Mifc, CmpdDitError> {
        let time = self.time;
        let (day, hour, min) = parse_time_str(&time)
            .ok_or_else(|| CmpdDitError::TimeCvrt(time))?;
        
        let dilution = self.dilution.unwrap_or(1.0);
        let value = self.value.map(|v| v * dilution);
        let exclude = self.flag.as_ref()
            .filter(|s| s.contains("O") || s.contains("W") || s.contains("F") )
            .map(|_| "X".to_string());
        
        Ok(Mifc {
            id: self.id,
            assay_plate_id: Some(self.group),
            assay_well_id: None,
            method: self.method,
            target: self.target,
            subtarget: None,
            sample_loc: self.location,
            day,
            hour,
            min,
            value,
            value_unit: self.value_unit,
            flag: self.flag,
            exclude,
            notes: self.note,
            replicate: self.replicate.map(|i| i as f32),
            xref: self.xref,
        })
    }
}

fn parse_time_str(time: &str) -> Option<(f64, f64, f64)> {
    let mut parts = time.split(".");
    let day  = parts.next()?.parse().ok()?;
    let hour = parts.next().map(str::parse).unwrap_or(Ok(0.0)).ok()?; 
    let min  = parts.next().map(str::parse).unwrap_or(Ok(0.0)).ok()?;

    Some((day, hour, min)) 
}
