# cmpdfmt

Format an input `.csv` file of TCTC analytical data into the MIFC format. Propogate special terms like "stock" to all chips that are in the same group as the special term.

## Format Overview
| Column Header        | Example Value | Description |
|----------------------|---------------|-------------|
| Group Indicator      | Low Dose      | A group for the chip; when a "special" chip is found, it's data is copied to all members of its group |
| Chip ID              | C001          | Chip id; this is where the "special term" goes (see below) |
| Time                 | 10.1.0        | d.h.m or, without periods, days |
| Method/Kit           | Mass Spec     | MPS-db method |
| Target/Analyte       | Caffeine      | MPS-db targert |
| Result               | 0.15          | float value of the result |
| Result Unit          | µM            | MPS-db unit |
| Dilution             | 1             | Modify the measured result (`Result * Dilution`) |
| Location             | effluent      | MPS-db sample location |
| Note (optional)      |               | Any notes about this sample |
| Flag (optional)      | F             | MPS-db flag; Flags `O`, `W`, and `F` cause the row to be excluded|
| Replicate (optional) |               | Indicate if there are multiple samples that are replicates of each other |
| TCTCxRef (optional)  |               | User defined |
| Cell Count           | 1e5           | float; Not used as of now |
| Sample Duration      | 1             | d.h.m or, without periods, days; Not used as of now |
| Sample Volume (µL)   | 100           | float; Not used as of now |s


## Special Terms
When certain terms are used for a chip id, the data in that row are propagated to all the other non-special chips in that group. 

By default, the special terms are:  
* `stock`
* `reservoir`

More can be specified with input flag `-t`/`--term` 

## Usage
```
USAGE:
    cmpdfmt [FLAGS] [OPTIONS] [--] [INPUT]...

FLAGS:
    -h, --help       Prints help information
        --stdout     Output the conversion of each file to stdout instead of writing to files
    -V, --version    Prints version information
    -v               Set the verbosity level (1, 2, or 3)

OPTIONS:
    -a, --append <append>          Append to input filename for output filename; defaults to "mifc"
    -t, --term <other_terms>...    Other, special propagating terms besides stock and reservoir
    -o, --out-dir <out_dir>        If present, directory in which output files are created

ARGS:
    <INPUT>...    Any number of input compound columnar csv files or directories containing those csv files
```
