# `usnjrnl`
Parses Windows $UsnJrnl files

This crate contains a library and a binary. If you only want to use the library in your crate, use `default-features=false` 
in your `Cargo.toml`:

```ini
[dependencies]
usnjrnl = {version="0.2.0", default-features=false }
```

## The library

### Usage

```rust
use usnjrnl::{UsnJrnlReader, CommonUsnRecord, UsnRecordData};

let reader = UsnJrnlReader::from("$UsnJrnl:$J")?;
for entry in reader.into_iter() {
    println!("{}: {}",
        entry.data.filename(),
        entry.data.reasons();
}
```

## `usnjrnl_dump`

### Usage

```
USAGE:
    usnjrnl_dump [FLAGS] <USNJRNL_FILE>

FLAGS:
    -b, --bodyfile    output as bodyfile instead of JSON
    -h, --help        Prints help information
    -V, --version     Prints version information

ARGS:
    <USNJRNL_FILE>    path to $UsnJrnl:$J file
```