use anyhow::{Result, anyhow};
use clap::{App, Arg};
use std::path::PathBuf;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ColorChoice};
use usnjrnl::{UsnJrnlReader, CommonUsnRecord, UsnRecordData};
use bodyfile::Bodyfile3Line;
use serde_json::json;

pub fn main() -> Result<()> {
    let _ = TermLogger::init(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto);
    
    let app = App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("USNJRNL_FILE")
                .help("path to $UsnJrnl:$J file")
                .required(true)
                .multiple(false)
                .takes_value(true),
        ).arg(
            Arg::with_name("BODYFILE_FORMAT")
                .short("b")
                .long("bodyfile")
                .help("output as bodyfile instead of JSON")
        );
    
    let matches = app.get_matches();
    let filename = match matches.value_of("USNJRNL_FILE") {
        None => {
            return Err(anyhow!("Missing filename for $UsnJrnl:$J file"));
        }
        Some(v) => PathBuf::from(v)
    };

    let reader = UsnJrnlReader::from(&filename)?;
    let formatter: Box<dyn RecordFormat> = if matches.is_present("BODYFILE_FORMAT") {
        Box::new(BodyfileFormatter{})
    } else {
        Box::new(JsonFormatter{})
    };
    for entry in reader.into_iter() {
        println!("{}", formatter.fmt(&entry));
    }

    Ok(())
}

trait RecordFormat {
    fn fmt(&self, record: &CommonUsnRecord) -> String;
}

struct BodyfileFormatter {}
impl RecordFormat for BodyfileFormatter {
    fn fmt(&self, record: &CommonUsnRecord) -> String {
        let message = format!("{} (UsnJrnl reason: {})",
                        record.data.filename(),
                        record.data.reason());
        let mut bf_line = Bodyfile3Line::new()
            .with_name(&message)
            .with_mtime(record.data.timestamp().timestamp());

        #[allow(irrefutable_let_patterns)]
        if let UsnRecordData::V2(ref v2record) = record.data {
            bf_line = bf_line.with_owned_inode(format!("{}", v2record.FileReferenceNumber.entry));
        }
        bf_line.to_string()
    }
}

struct JsonFormatter {}
impl RecordFormat for JsonFormatter {
    fn fmt(&self, record: &CommonUsnRecord) -> String {
        let json = json!({
            "timestamp": record.data.timestamp(),
            "filename": record.data.filename(),
            "reason": record.data.reason().to_string(),
        });
        json.to_string()
    }
}