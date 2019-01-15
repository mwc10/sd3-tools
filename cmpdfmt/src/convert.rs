use std::collections::{HashMap, HashSet, BTreeSet};
use std::path::{Path, PathBuf};
use std::io::{Write};
use log::{error, warn, info, debug};
use failure::{Error, ResultExt, Fail};
use sd3::{CmpdDit, Mifc};
use crate::{output};

/// A HashSet that contains the various data points with special chip ids 
/// whose data are duplicated to any chips that share the same group.  
/// By default, the special IDs are "stock" and "reservoir", though
/// others can be passed to the `new` method when making a new set.
#[derive(Debug)]
struct PropGroups<'opt> (HashSet<&'opt str>);
impl<'opt> PropGroups<'opt> {
    fn new(others: impl Iterator<Item = &'opt str>) -> Self {
        let groups = ["stock", "reservoir"]
            .iter()
            .map(|s| *s)
            .chain(others)
            .collect();

        PropGroups(groups)
    }

    fn get(&self, group: &str) -> Option<&'opt str> {
        self.0.get(group).map(|s| *s)
    }
}

/// A map of a group name for a given chip with info about that group.
/// The info includes things like the chips in that group and any special 
/// propagating data.
type ChipGroups<'opt> = HashMap<String, GroupInfo<'opt>>;

/// The info about a group that is stored in a `ChipGroups`. It contains
/// a collection of the chips in this group, as well as any info that 
/// should be propagated to all those chips.
#[derive(Debug)]
struct GroupInfo<'opt> {
    chips: BTreeSet<String>,
    // Map<PropGroupName, Vec<TimePoint MIFC Data>>
    stored_info: Option<HashMap<&'opt str, Vec<Mifc>>>,
}

impl<'opt> GroupInfo<'opt> {
    fn new() -> Self {
        GroupInfo {
            chips: BTreeSet::new(),
            stored_info: None,
        }
    }
    /// Add a new `info` under the name `group` to be propagated later 
    fn propagate(&mut self, group: &'opt str, info: Mifc) {
        self.stored_info.get_or_insert_with(|| HashMap::with_capacity(2))
            .entry(group)
            .or_insert(Vec::with_capacity(4))
            .push(info);
    }
    /// store another chip id
    fn add_chip(&mut self, id: &str) {
        self.chips.insert(id.to_string());
    }
}

/// Helper error type to handle recoverable or unrecoverable de/serialization errors
#[derive(Debug, Fail)]
enum ConversionErr {
    #[fail(display = "recoverable conversion error")]
    Recoverable(#[cause] Error),
    #[fail(display = "unrecoverable conversion error")]
    NotRecoverable(#[cause] Error),
}
/// Helper functions for mapping to a `ConversionErr`
fn recoverable_err<E: Into<Error>>(e: E) -> ConversionErr {ConversionErr::Recoverable(e.into())}
fn unrecoverable_err<E: Into<Error>>(e: E) -> ConversionErr {ConversionErr::NotRecoverable(e.into())}

/// The key function that converts an `Iterator` of paths to CSV CMPD MIFC files into proper output MIFC files 
pub fn cmpd_csv_to_mifc<'i>(files: impl Iterator<Item = PathBuf> + 'i, options: &crate::Opt) -> Result<(), Error> 
{
    let other_terms = options.other_terms.iter().map(String::as_str);
    let prop_groups = PropGroups::new(other_terms);

    for path in files {
        match convert_file(&path, options, &prop_groups) {
            Err(ConversionErr::Recoverable(e)) => {
                error!("skipping file <{:?}> due to:", &path);
                errlog::print_chain(&e);
                continue;
            },
            e @ Err(ConversionErr::NotRecoverable(_)) => {
                e.context("stopping conversion of all files")?;
            }
            _ => (),
        };
    }

    debug!("{:?}", &prop_groups);

    Ok(())
}

/// Handle the conversion of an individual CSV file 
fn convert_file<'opt, 'f>(
    path: &'f Path, 
    options: &'opt crate::Opt, 
    propgrps: &'f PropGroups<'opt>
) -> Result<(), ConversionErr>
{
    let append_str = options.append.as_ref().map(|s| s.as_str()).unwrap_or("mifc");
    let output_dir = options.out_dir.as_ref().map(|o| o.as_path());
    let use_stdout = options.stdout;

    info!("reading {:?}", &path);

    let mut csv_rdr = csv::Reader::from_path(&path)
        .context(format!("couldn't open input for reading; skipping file <{:?}>", &path))
        .map_err(recoverable_err)?;
    
    let mut groups = ChipGroups::new();
    // TODO: return output file path as well? for cleanup?
    let wtr = output::get_output_wtr(use_stdout, &output_dir, &path, &append_str)
        .context(format!("couldn't open output; skipping file <{:?}>", &path))
        .map_err(recoverable_err)?;
    let mut wtr = csv::Writer::from_writer(wtr);

    for result in csv_rdr.deserialize() {
        let record: CmpdDit = match result {
            Ok(r)  => r,
            Err(e) => {
                warn!("couldn't deserialize row");
                errlog::warn_chain(&e.into());
                continue;
            }
        };

        match write_record(record, &mut wtr, &propgrps, &mut groups) {
            Err(ConversionErr::Recoverable(e)) => {
                warn!("skipping row in <{:?}>", &path);
                errlog::warn_chain(&e);
                continue;
            },
            Err(ConversionErr::NotRecoverable(e)) => {
                return Err(ConversionErr::Recoverable(e));
            }
            _ => (),
        };
    }
    // propagate various other collected data points, if needed
    write_prop_rows(&mut wtr, groups).map_err(recoverable_err)?;

    Ok(())
}

/// Write out one record from the input CSV file to the ouput, 
/// and save any important info about that record into the `&mut ChipGroups` struct  
fn write_record<'opt: 'f, 'f: 'r, 'r, W: Write>(
    record: CmpdDit, 
    output: &'r mut csv::Writer<W>,
    prop_grp: &'f PropGroups<'opt>, 
    chip_grps: &'r mut ChipGroups<'opt>
) -> Result<(), ConversionErr> 
{
    let r_group = record.group();
    let r_id = record.chip_id();
    let group_info = if let Some(info) = chip_grps.get_mut(r_group) {
        info
    } else {
        chip_grps.entry(r_group.clone()).or_insert(GroupInfo::new())
    };

    /* A chip with an id of "stock"/"reservoir"/etc. means that the info for 
    ** that chip is meant to be applied to all chips in the a group */
    if let Some(group) = prop_grp.get(&r_id) {
        let prop_mifc = record.into_mifc()
            .context("converting a propagating group into MIFC format")
            .map_err(unrecoverable_err)?;
        
        group_info.propagate(group, prop_mifc); 
    } 
    /* otherwise, add chip id to the group map and convert the row */
    else {
        group_info.add_chip(&r_id);
        let mifc = record.into_mifc()
            .context("converting a standard row into MIFC format")
            .map_err(recoverable_err)?;
        
        output.serialize(&mifc)
            .context("writing serialized MIFC for normal row")
            .map_err(recoverable_err)?;
    }

    Ok(())
}

/// Propagating any information stored from propagating rows into the output
/// CSV file for each chip that should have that info
fn write_prop_rows<W: Write>(output: &mut csv::Writer<W>, groups: ChipGroups) -> Result<(), Error> {
    use std::fmt::Write;

    for (_group_name, group_info) in groups.into_iter() {
        if group_info.stored_info.is_none() { continue; }
        let info = group_info.stored_info.unwrap();
        let chips = group_info.chips;

        for (_propgrp, datapoints) in info {
            for mut point in datapoints {
                for id in chips.iter() {
                    // be cheeky and avoid allocations by reusing the id String
                    point.id.clear();
                    point.id.write_str(id)?;
                    output.serialize(&point)
                        .context("writing propagating chip group data to output")?; // TODO -> match
                }
            }
        }
    }

    Ok(())
}
