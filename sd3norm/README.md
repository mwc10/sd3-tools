# SD3 Normalization Tool

Input an any number of "SD3" `.xlsx` data files with inline normalization info--or directories containing data files--and normalize that data into an output `.csv` file.

## Installation Instructions
1) [Install rust](http://rustup.rs)
2) Clone this repository and navigate to the clonse
3) `cargo install`

## Normalization Fields
After the standard SD3 fields, this tool looks for these columns:

| Duration Sample Collection (days) | Duration Sample Collection (hours) | Duration Sample Collection (minutes) | Sample Volume | Sample Volume Unit | Estimated Cell Number |
|-----------------------------------|------------------------------------|--------------------------------------|---------------|--------------------|-----------------------|
| Float                             | Float                              | Float                                | Float         | String             | Float                 |
| 1                                 | 0                                  | 0                                    | 300           | uL                 | 80,000                |

## Some Rows are not Normalized
* Exclude field is not empty
* No Value
* Unexpected input in either the SD3 columns or the normalization columns

## Usage
```
sd3norm 0.5.1
Mike C. <mwc10>
Read an SD3 (MIFC + normalization info) excel workbook and create one normalized MIFC CSV for each sheet

USAGE:
    sd3norm.exe [FLAGS] [OPTIONS] [INPUT]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Print debug info based on the number of "v"s passed

OPTIONS:
    -a, --append <append>      Append to INPUT for output, defaults to "normalized"
    -d, --out-dir <out_dir>    Directory to create output file(s) in

ARGS:
    <INPUT>...    Any number of input sd3-formatted excel files or directories containing excel files

```