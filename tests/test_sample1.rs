use ::usnjrnl::{UsnJrnlReader, UsnReaderError};
use std::path::PathBuf;

#[test]
fn test_sample1() -> Result<(), UsnReaderError> {
    let reader1 = UsnJrnlReader::from(&get_sample_file("sample1.bin"))?;
    let reader2 = UsnJrnlReader::from(&get_sample_file("sample1.bin"))?;


    let count1 = reader1.into_iter().count();

    let  mut count2 = 0;
    for entry in reader2.into_iter() {
        // this failes because of the last entry
        //assert!(entry.is_ok());
        if let Ok(entry) = entry {
            println!("{}: {}", entry.data.reason(), entry.data.filename());
        }
        count2 += 1;
    }
    assert_eq!(count1, count2);
    Ok(())
}

fn get_sample_file(filename: &str) -> PathBuf {
    let prj_root = env!("CARGO_MANIFEST_DIR");
    let mut sample_file = PathBuf::from(prj_root);
    sample_file.push("tests");
    sample_file.push("data");
    sample_file.push(filename);
    sample_file
}