# `usnjrnl`
Parses Windows $UsnJrnl files

This crate contains a library and a binary. If you only want to use the library in your crate, use `default-features=false` 
in your `Cargo.toml`:

```ini
[dependencies]
usnjrnl = {version="0.3.0", default-features=false }
```

## Installation

```shell
cargo install usnjrnl
```

## Usage 

### `usnjrnl_dump` binary

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

I suggest to always correlate MFT entry numbers to entries in a real `$MFT` file. This can be done automatically with <https://github.com/janstarke/mft2bodyfile>.

### `usnjrnl` library

```rust
use usnjrnl::{UsnJrnlReader, CommonUsnRecord, UsnRecordData};

let reader = UsnJrnlReader::from("$UsnJrnl:$J")?;
for entry in reader.into_iter() {
    match entry {
        Ok(e) => {
            println!("{}: {}",
                e.data.filename(),
                e.data.reasons();
        }
        Err(why) => {
            log::error!("{}", why);
        }
    }
}
```
