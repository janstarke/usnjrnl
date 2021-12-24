use crate::{CommonUsnRecord, UsnReaderError};
use std::io::{Result, Read, Seek, Cursor, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use memmap::{Mmap, MmapOptions};

pub struct UsnJrnlReader {
    data: Mmap,
}

impl UsnJrnlReader {
    pub fn from(file_path: &PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Self {
            data: mmap
        })
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> UsrJrnlIterator<Cursor<&[u8]>> {
        UsrJrnlIterator::from(
            Cursor::new(&self.data[..])
        )
    }
}

impl IntoIterator for UsnJrnlReader {
    type Item = CommonUsnRecord;
    type IntoIter = UsrJrnlIterator<Cursor<Mmap>>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(Cursor::new(self.data))
    }
}

fn next_from_data<RS>(data: &mut RS, index: &mut usize) -> std::result::Result<CommonUsnRecord, UsnReaderError> where RS: Read + Seek {

    let stream_len = match data.seek(SeekFrom::End(0)) {
        Ok(size) => size as usize,
        Err(why) => {
            return Err(UsnReaderError::from(why));
        }
    };

    if *index < stream_len {
        match CommonUsnRecord::from(data, index) {
            Ok(record) => {
                *index += record.header.RecordLength as usize;
                Ok(record)
            }

            Err(UsnReaderError::NoMoreData) => Err(UsnReaderError::NoMoreData),

            Err(why) => {
                log::error!("error while parsing logfile: {}", why);
                Err(why)
            }
        }
    } else {
        Err(UsnReaderError::NoMoreData)
    }
}

pub struct UsrJrnlIterator<RS> where RS: Read + Seek {
    data: RS,
    current_offset: usize,
    last_error: Option<UsnReaderError>
}

impl<RS> UsrJrnlIterator<RS> where RS: Read + Seek {
    pub fn from(data: RS) -> Self {
        Self {
            data,
            current_offset: 0,
            last_error: None
        }
    }
}

impl<RS> Iterator for UsrJrnlIterator<RS> where RS: Read + Seek {
    type Item = CommonUsnRecord;
    fn next(&mut self) -> Option<Self::Item> {
        match next_from_data(&mut self.data, &mut self.current_offset) {
            Ok(result) => Some(result),
            Err(why) => {
                self.last_error = Some(why);
                None
            }
        }
    }
}