use serde_derive::{Deserialize, Serialize};
use failure::{Fail};
use log::{trace};
use units::{SIUnit, self};

#[derive(Debug, Fail)]
/// Errors that can occur during normalization of a `MifcNorm` into a `Mifc`
pub enum MifcNormError {
    #[fail(display = "row had a non-empty Exclude column")]
    Excluded,
    #[fail(display = "row did not have associated normalization info columns")]
    NoInfo,
    #[fail(display = "row did not have an entered Value")]
    NoValue,
    #[fail(display = "row did not have an entered Value Unit")]
    NoValueUnit,
}
//TODO: Deserialize optional string fields with a null || "" = None checking function

/// MIFC fields with additional normalization info. This struct can be used 
/// to create a new MIFC row that has been normalized based on that additional info.
#[derive(Debug, Serialize, Deserialize)]
pub struct MifcNorm {
    #[serde(flatten)]
    mifc: Mifc,
    #[serde(flatten)]
    normal_info: Option<Normalization>,
}

impl MifcNorm {
    /// Transform a `MifcNorm` into a `Mifc` by using the normalization information 
    /// contained with the `MifcNorm` `struct`.
    pub fn into_normalized(self) -> Result<Mifc, MifcNormError> {
        if let Some(ref f) = self.mifc.exclude {
            if f != "" { return Err(MifcNormError::Excluded) }
        }
        let value = self.mifc.value.ok_or(MifcNormError::NoValue)?;
        let value_unit = self.mifc.value_unit.ok_or(MifcNormError::NoValueUnit)?;
        let info = self.normal_info.ok_or(MifcNormError::NoInfo)?;

        let sample_time = info.calc_sample_time();
        let norm_val = to_ngday_millioncells(value, value_unit, &info);

        let mut normalized_mifc = self.mifc;
        let note = format!("Normalized from {v:.4} {vu} by a {s} {su} sample over {d} {ds} with an estimated {c} cells ", 
            v = value, vu = value_unit,
            s = info.sample_volume, su = info.sample_vol_unit,
            d = sample_time, ds = if sample_time > 1.0 {"days"} else {"day"},
            c = info.cell_count
        );

        normalized_mifc.value = Some(norm_val);
        normalized_mifc.value_unit = Some(SIUnit::ng_day_millioncells);        
        normalized_mifc.notes = if let Some(mut n) = normalized_mifc.notes {
            if &n != "" { n.push_str(" || "); }
            n.push_str(&note);
            Some(n)
        } else {
            Some(note)
        };

        Ok(normalized_mifc)
    }
}

/// Necessary MIFC fields
#[derive(Debug, Serialize, Deserialize)]
pub struct Mifc {
    #[serde(rename = "Chip ID")]
    pub id: String,
    #[serde(rename = "Assay Plate ID")]
    pub assay_plate_id: Option<String>,
    #[serde(rename = "Assay Well ID")]
    pub assay_well_id: Option<String>,
    #[serde(rename = "Method/Kit")]
    pub method: String,
    #[serde(rename = "Target/Analyte")]
    pub target: String,
    #[serde(rename = "Subtarget")]
    pub subtarget: Option<String>,
    #[serde(rename = "Sample Location")]
    pub sample_loc: String,
    #[serde(rename = "Day")]
    pub day: f64,
    #[serde(rename = "Hour")]
    pub hour: f64,
    #[serde(rename = "Minute")]
    pub min: f64,
    #[serde(rename = "Value")]
    pub value: Option<f64>,
    #[serde(rename = "Value Unit")]
    pub value_unit: Option<SIUnit>, 
    #[serde(rename = "Caution Flag")]
    pub flag: Option<String>,
    #[serde(rename = "Exclude")]
    pub exclude: Option<String>,
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
    #[serde(rename = "Replicate")]
    pub replicate: Option<f32>,
    #[serde(rename = "Cross Reference")]
    pub xref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Normalization {
    #[serde(rename = "Duration Sample Collection (days)")]
    sample_days: f64,
    #[serde(rename = "Duration Sample Collection (hours)")]
    sample_hours: f64,
    #[serde(rename = "Duration Sample Collection (minutes)")]
    sample_minutes: f64,
    #[serde(rename = "Sample Volume")]
    sample_volume: f64,
    #[serde(rename = "Sample Volume Unit")]
    sample_vol_unit: SIUnit,
    #[serde(rename = "Estimated Cell Number")]
    cell_count: f64,
}

impl Normalization {
    /// Calculate the duration of the sample in terms of days
    #[inline]
    fn calc_sample_time(&self) -> f64 {
        self.sample_days 
        + (self.sample_hours/24.0) 
        + (self.sample_minutes/(24.0*60.0))
    }
}

fn to_ngday_millioncells(val: f64, val_unit: SIUnit, norm: &Normalization) -> f64
{
    use self::SIUnit::*;

    let &Normalization{cell_count: cells, sample_volume: vol, sample_vol_unit: vol_unit, ..} = norm;

    let days = norm.calc_sample_time();
    let si_val = units::convert((val, val_unit), g_l).unwrap();
    let si_vol = units::convert((vol, vol_unit), l).unwrap();
    trace!("conc: {:.5} {} to SI {:.5} {}", val, val_unit, si_val, g_l);
    trace!("vol: {:.5} {} to SI {:.5} {}",  vol, vol_unit, si_vol, l);

    // first go from the concentration (g/L) and sample volume (L) 
    // into nanograms/day/cell
    let made_ng = units::convert((si_val * si_vol, g), ng).unwrap();
    trace!("produced ng: {:.5} over {:.3} day(s)", made_ng, days);
    let ngdaycell = made_ng / days / cells;
    // now, multiple by 10^6 to make rate by million cells 
    ngdaycell * 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::double_comparable;
    use units::SIUnit::*;

    struct Norm {
        val: f64,
        val_unit: SIUnit,
        info: Normalization,
    }

    static INPUTS: [Norm; 10] = [
        Norm {
            val: 153.914,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 16768.0,
            }
        },
        Norm {
            val: 1360.2953,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 16768.0,
            }
        },
        Norm {
            val: 1071.288,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 300.0,
                sample_vol_unit: ul,
                cell_count: 80000.0,
            }
        },
        Norm {
            val: 1543.054,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 484321.0,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 2.0,
                sample_hours: 5.0,
                sample_minutes: 0.0,
                sample_volume: 500.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 15.9,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 20.0,
                sample_minutes: 2.0,
                sample_volume: 100.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 0.87,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 10.0,
                sample_minutes: 30.0,
                sample_volume: 0.1,
                sample_vol_unit: ml,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 542.0,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 3.0,
                sample_hours: 15.0,
                sample_minutes: 1.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 20000.0,
            }
        },
        Norm {
            val: 12.0556,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 0.1,
                sample_vol_unit: ml,
                cell_count: 20000.0,
            }
        },
        Norm {
            val: 0.00465,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 2.0,
                sample_minutes: 30.0,
                sample_volume: 0.01,
                sample_vol_unit: l,
                cell_count: 20000.0,
            }
        },
    ];

    static OUTPUTS: [f64; 10] = [
        1835.801527,
        16224.89623,
        4017.33,
        61722160.0,
        21931516981.0,
        380965.0582,
        39771.42857,
        1.494886037,
        0.060278,
        0.02232,
    ];

    #[test]
    fn ng_day_cell_normalization() {
        const PERCENT_TOLERANCE: f64 = 0.001;

        let all_equal = INPUTS.iter()
            .map(|i| to_ngday_millioncells(i.val, i.val_unit, &i.info))
            .zip(OUTPUTS.iter())
            .enumerate()
            .inspect(|(i, (c, e))|
                println!("\nSet #{}\ncalculated: {} | expected: {}", i, c, e)
            )
            .all(|(_i,(c,e))|
                double_comparable(c, *e, PERCENT_TOLERANCE)
            );

        assert!(all_equal);
    }
}
