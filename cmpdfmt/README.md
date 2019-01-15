# cmpdfmt

Format an input `.csv` file of TCTC analytical data into the MIFC format. Propogate special terms like "stock" to all chips that are in the same group as the special term.

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
