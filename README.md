

# TTTR Toolbox
The fastest streaming algorithms for your TTTR data.

TTTR Toolbox can be used as a standalone Rust library. If you do most of your data
analysis in Python you may prefer to check Trattoria, a wrapper library for this
crate.

## Project Goals
- Single threaded performance
- Ease of extensibility

## Algorithms available
- second order autocorrelation (g2)
- intensity time trace
- record number time trace
- zero delay finder

## Supported file and record formats
- PicoQuant PTU
  - PHT2
  - HHT2_HH1
  - HHT2_HH2
  - HHT3_HH@

If you want support for more record formats and file formats please ask for it.
At the very least we will need the file format specification and a file with some
discernible features to test the implementation.

## Examples
```rust
pub fn main() {
    let filename = PathBuf::from("/Users/garfield/Downloads/20191205_Xminus_0p1Ve-6_CW_HBT.ptu");
    let ptu_file = File::PTU(PTUFile::new(filename).unwrap());
    // Unwrap the file so we can print the header
    let File::PTU(f) = &ptu_file;
    println!("{}", f);

    let params = G2Params {
        channel_1: 0,
        channel_2: 1,
        correlation_window: 50_000e-12,
        resolution: 600e-12,
        start_record: None,
        stop_record: None,
    };
    let g2_histogram = g2(&ptu_file, &params).unwrap();
    println!("{:?}", g2_histogram.hist);
}
```
