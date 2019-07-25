use std::collections::HashMap;

struct RawInput<'i> {
    chip: &'i str,
    control: bool,
    day: f64,
    notes: Option<&'i str>,
    /// Map between target name and info about that target
    values: HashMap<&'i str, ValueRecord<'i>>,
}

struct ValueRecord<'i> {
    target: &'i str,
    value: f64,
    flags: Option<&'i str>,
    exclude: bool,
    notes: Option<&'i str>,
}

/// Column Id for each of a target's subflieds
#[derive(Debug, Default)]
pub struct TargetColIds {
    value: usize,
    flags: Option<usize>,
    exclude: Option<usize>,
    notes: Option<usize>,
}

/// Map between target name and column ids for Value, Flags, Exclude, Notes
pub type TargetColLUT = HashMap<String, ColIds>;

/// Fixed Necessary fields Ids
#[derive(Debug)]
pub struct ColIds {
    chip: usize,
    control: usize,
    time: usize,
    time_unit: Time,
    notes: Option<usize>,
}

/// Columns for required fields and target fields
#[derive(Debug)]
pub struct ParsedHeader {
    required: ColIds,
    targets: TargetColLUT,
}

#[derive(Debug, failure::Fail)]
pub enum HeaderParseError {
    #[fail(display = "Missing required column '{}'", _0)]
    MissingRequired(&'static str),
    #[fail(display = "No targets columns")]
    NoTargets,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Time {
    Days,
    Hours,
    Minutes,
}

pub fn parse_header(hdr: &csv::StringRecord) -> Result<ParsedHeader, HeaderParseError> {
    use HeaderParseError::*;

    let mut chip = None;
    let mut control = None;
    let mut time = None;
    let mut time_unit = Time::Days;
    let mut notes = None;
    let mut targets = TargetColLUT::new();

    for (i, col) in hdr.iter().enumerate() {
        dbg!(i, col);

        match col.trim() {
            "Chip ID" => {
                chip = Some(i);
                continue;
            }
            "Control Chip" => {
                control = Some(i);
                continue;
            }
            "Notes" => {
                notes = Some(i);
                continue;
            }
            _ => (),
        }

        if col.starts_with("Time") {
            time = Some(i);
            // default to days
            time_unit = get_time_unit(col).unwrap_or(Time::Days);
            continue;
        }

        // otherwise, must be a target or its associated fields
        dbg!(get_target_name(col));
    }

    chip.ok_or(MissingRequired("Chip ID"))
        .and_then(|chip| {
            control
                .ok_or(MissingRequired("Control Chip"))
                .and_then(|control| {
                    time.ok_or(MissingRequired("Time")).and_then(|time| {
                        Ok(ColIds {
                            chip,
                            time,
                            notes,
                            time_unit,
                            control,
                        })
                    })
                })
        })
        .map(move |required| ParsedHeader { required, targets })
}

fn get_time_unit(t: &str) -> Option<Time> {
    let start = t.find('[');
    let end = t.find(']');

    start
        .and_then(|s| end.and_then(|e| t.get(s + 1..e)))
        .and_then(|unit| match unit {
            "day" | "days" => Some(Time::Days),
            "hr" | "hour" | "hours" => Some(Time::Hours),
            "min" | "minute" | "minutes" => Some(Time::Minutes),
            _ => None,
        })
}

/// This enforces the "My Target" or "My Target [Optional]" field
fn get_target_name(raw: &str) -> &str {
    raw.find('[')
        .and_then(|optional_start| raw.get(..optional_start))
        .map(|t| t.trim())
        .unwrap_or_else(|| raw.trim())
}
