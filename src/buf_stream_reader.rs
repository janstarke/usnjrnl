
use std::io::{Result, Read, Seek, ErrorKind, Error, Cursor, SeekFrom};
pub struct BufStreamReader<R> where R: Read {
    reader: R,
    offset: u64,
    buffer_size: usize,
    cursor: Cursor<Vec<u8>>,
}

impl<R> BufStreamReader<R> where R: Read {
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

    fn read_next_buffer(&mut self) -> Result<()> {
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

impl<R> Read for BufStreamReader<R> where R: Read {
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

impl<R> Seek for BufStreamReader<R> where R: Read {
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64> {
        match seek_from {
            SeekFrom::Start(pos) => {
                Ok(self.cursor.seek(SeekFrom::Start(pos - self.offset))? + self.offset)
            }

            SeekFrom::Current(pos) => {
                Ok(self.cursor.seek(SeekFrom::Current(pos))? + self.offset)
            }

            // We don't know where the end of a stream is, so this cannot be implemented
            SeekFrom::End(_) => {
                unimplemented!();
            }
        }
    }
}