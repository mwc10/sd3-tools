# ImgQC Tool

## Metadata files
The tool looks for the following four files in a user supplied directory:
* `MPS Database Targets.csv`
* `MPS Database Methods.csv`
* `MPS Database Locations.csv`
* `MPS Database Units.csv`

For the Chip metadata, the user can enter a path to a `CSV` file, or place a file named `MPS Database Chips.csv` with the other metadata files

### Metadata File Formats
#### `MPS Database Targets.csv`
Required Column: `Target`

Download from the [targets page](https://mps.csb.pitt.edu/assays/target/) and rename the file. No column changes are necessary.

#### `MPS Database Methods.csv`
Required Column: `Method`

Download from the [methods page](https://mps.csb.pitt.edu/assays/method/) and rename the file. No column changes are necessary.

#### `MPS Database Locations.csv`
Required Column: `Location`

Download from the [locations page](https://mps.csb.pitt.edu/assays/locations/) and rename the file. No column changes are necessary.

#### `MPS Database Units.csv`
Required Column: `Unit`

Download from the [units page](https://mps.csb.pitt.edu/assays/units/) and rename the file. No column changes are necessary.

#### Chip Metadata
Required Column: `Name`
Download from the study of interest. No column renaming should have to be done.
