use ::usnjrnl::{UsnJrnlReader, UsnReaderError};
use std::path::PathBuf;

#[test]
fn test_sample1() -> Result<(), UsnReaderError> {
    let reader = UsnJrnlReader::from(&get_sample_file("sample1.bin"))?;

    let count1 = reader.iter().count();

    let  mut count2 = 0;
    for entry in reader.into_iter() {
        println!("{}: {}", entry.data.reason(), entry.data.filename());
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