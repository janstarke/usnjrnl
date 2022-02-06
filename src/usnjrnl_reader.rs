use crate::{CommonUsnRecord, UsnReaderError};
use std::io::{Result, Read, Seek, Cursor, SeekFrom, ErrorKind, Error};
use std::fs::File;
use std::path::PathBuf;

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
    type IntoIter = UsrJrnlIterator<ForwardBufferedReader<Box<dyn Read>>>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(ForwardBufferedReader::new(self.reader).unwrap())
    }
}

pub struct ForwardBufferedReader<R> where R: Read {
    reader: R,
    offset: u64,
    buffer_size: usize,
    cursor: Cursor<Vec<u8>>,
}

impl<R> ForwardBufferedReader<R> where R: Read {
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        // already read the first buffer:
        let (bytes, cursor) = Self::initialize_buffer(&mut reader)?;

        Ok(Self {
            reader,
            cursor,
            buffer_size: bytes,
            offset: 0,
        })
    }

    pub fn read_next_buffer(&mut self) -> Result<()> {
        let (bytes, cursor) = Self::initialize_buffer(&mut self.reader)?;
        self.offset += self.buffer_size as u64;
        self.cursor = cursor;
        self.buffer_size = bytes;
        Ok(())
    }

    fn initialize_buffer(reader: &mut R) -> Result<(usize, Cursor<Vec<u8>>)> {
        let mut buffer = vec![0; 4096];
        let bytes = reader.read(&mut buffer[..])?;
        if bytes == 0 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "read 0 bytes"));
        }
        Ok((bytes, Cursor::new(buffer)))
    }
}

impl<R> Read for ForwardBufferedReader<R> where R: Read {
    fn read(&mut self, dst: &mut [u8]) -> Result<usize> {
        let mut bytes_read = 0;
        loop {
            match self.cursor.read(&mut dst[bytes_read..]) {
                Ok(bytes) => {
                    bytes_read += bytes;
                    if bytes_read == dst.len() {
                        return Ok(bytes_read)
                    }
                    assert!(bytes_read < dst.len());
                    self.read_next_buffer()?;
                }
                Err(why) => match why.kind() {
                    ErrorKind::UnexpectedEof => {
                        self.read_next_buffer()?;
                    }
                    _ => {
                        return Err(why);
                    }
                }
            }
        }
    }
}

impl<R> Seek for ForwardBufferedReader<R> where R: Read {
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64> {
        match seek_from {
            SeekFrom::Start(pos) => {
                Ok(self.cursor.seek(SeekFrom::Start(pos - self.offset))? + self.offset)
            }

            SeekFrom::End(_) => {
                unimplemented!();
            }

            SeekFrom::Current(pos) => {
                Ok(self.cursor.seek(SeekFrom::Current(pos))? + self.offset)
            }
        }
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