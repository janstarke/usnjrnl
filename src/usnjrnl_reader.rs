use crate::{CommonUsnRecord, UsnReaderError};
use std::io::{Result, Read, Seek, ErrorKind, Error};
use std::fs::File;
use std::path::PathBuf;
use buf_stream_reader::BufStreamReader;

#[cfg(feature = "gzip")]
use flate2::read::GzDecoder;

pub struct UsnJrnlReader {
    reader: Box<dyn Read>,
}

impl UsnJrnlReader  {
    pub fn from(file_path: &PathBuf) -> Result<Self> {
        Ok(Self {
            reader: Self::open_file(file_path)?
        })
    }

    fn open_file(file_path: &PathBuf) -> Result<Box<dyn Read>> {
        let file = File::open(file_path)?;

        #[cfg(feature = "gzip")]
        match file_path.file_name() {
            Some(filename) => {
                if filename.to_string_lossy().ends_with(".gz") {
                    return Ok(Box::new(GzDecoder::new(file)));
                }
            }
            None => {
                return Err(Error::new(ErrorKind::InvalidInput, "missing filename"))
            }
        }

        Ok(Box::new(file))
    }
}


impl IntoIterator for UsnJrnlReader {
    type Item = std::result::Result<CommonUsnRecord, UsnReaderError>;
    type IntoIter = UsrJrnlIterator<BufStreamReader<Box<dyn Read>>>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(BufStreamReader::new(self.reader, 4096).unwrap())
    }
}

pub struct UsrJrnlIterator<RS> where RS: Read + Seek {
    data: RS,
}

impl<RS> UsrJrnlIterator<RS> where RS: Read + Seek {
    pub fn from(data: RS) -> Self {
        Self {
            data
        }
    }
}

impl<RS> Iterator for UsrJrnlIterator<RS> where RS: Read + Seek {
    type Item = std::result::Result<CommonUsnRecord, UsnReaderError>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_record = CommonUsnRecord::from(&mut self.data);
        if let Err(UsnReaderError::NoMoreData) = next_record {
            None
        } else {
            Some(next_record)
        }
    }
}