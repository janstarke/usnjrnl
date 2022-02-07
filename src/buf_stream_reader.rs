
use std::io::{Result, Read, Seek, ErrorKind, Error, Cursor, SeekFrom};

/// This struct provides a buffered access to a [`Read`](std::io::Read) object
/// with a limited [`Seek`](std::io::Seek) implementation.
/// 
/// # Seeking backward as possible as far as there are data in the current buffer
/// ```rust
/// use std::io::{Cursor, Read, Seek, SeekFrom};
/// use usnjrnl::buf_stream_reader::BufStreamReader;
/// # let mut arr: [u8; 256] = [0; 256];  
/// # for (elem, val) in arr.iter_mut().zip(0..=255) { *elem = val; }
/// let cursor = Cursor::new(&arr); // points to array with values from \x00 .. \xff
/// let mut reader = BufStreamReader::new(cursor, 16).unwrap();
/// 
/// let mut buffer: [u8; 7] = [0; 7];
/// 
/// /* straightly reading 7 bytes works */
/// assert_eq!(reader.read(&mut buffer).unwrap(), 7);
/// assert_eq!(&buffer, &arr[0..7]);
/// 
/// /* seeking backwards inside the current buffer */
/// assert!(reader.seek(SeekFrom::Current(-4)).is_ok());
/// assert_eq!(reader.read(&mut buffer).unwrap(), 7);
/// assert_eq!(&buffer, &arr[3..10]);
/// ```
/// 
/// # Seeking backwards is not possible if the destination is not within of behind the current buffer
/// ```rust
/// # use std::io::{Cursor, Read, Seek, SeekFrom};
/// # use usnjrnl::buf_stream_reader::BufStreamReader;
/// # let mut arr: [u8; 256] = [0; 256];  
/// # for (elem, val) in arr.iter_mut().zip(0..=255) { *elem = val; }
/// let cursor = Cursor::new(&arr); // points to array with values from \x00 .. \xff
/// let mut reader = BufStreamReader::new(cursor, 16).unwrap();
/// 
/// let mut buffer: [u8; 7] = [0; 7];
/// assert!(reader.seek(SeekFrom::Start(96)).is_ok());
/// assert!(reader.seek(SeekFrom::Start(95)).is_err());
/// ```
pub struct BufStreamReader<R> where R: Read {
    reader: R,
    offset: u64,
    buffer_size: usize,
    bytes_in_buffer: usize,
    cursor: Cursor<Vec<u8>>,
}

impl<R> BufStreamReader<R> where R: Read {
    pub fn new(mut reader: R, buffer_size: usize) -> Result<Self> {
        // already read the first buffer:
        let (bytes, cursor) = Self::initialize_buffer(&mut reader, buffer_size)?;

        Ok(Self {
            reader,
            cursor,
            buffer_size,
            bytes_in_buffer: bytes,
            offset: 0,
        })
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    fn read_next_buffer(&mut self) -> Result<()> {
        let (bytes, cursor) = Self::initialize_buffer(&mut self.reader, self.buffer_size)?;
        self.offset += self.bytes_in_buffer as u64;
        self.cursor = cursor;
        self.bytes_in_buffer = bytes;
        Ok(())
    }

    fn initialize_buffer(reader: &mut R, buffer_size: usize) -> Result<(usize, Cursor<Vec<u8>>)> {
        let mut buffer = vec![0; buffer_size];
        let bytes = reader.read(&mut buffer[..])?;
        if bytes == 0 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "read 0 bytes"));
        }
        Ok((bytes, Cursor::new(buffer)))
    }

    /// jump a certain number of blocks forward
    fn seek_until_position(&mut self, position_in_buffer: u64) -> Result<u64> {
        if (position_in_buffer as usize) < self.bytes_in_buffer {
            Ok(position_in_buffer)
        } else {
            let offset_in_buffer = position_in_buffer % self.buffer_size as u64;
             
            // One ot the buffers to skip has already been read, so this can be subtracted.
            let skip_buffers = ((position_in_buffer - offset_in_buffer) / self.buffer_size as u64) - 1;

            // Also, the destination buffer will not be skipped, so we subtract is also.
            if skip_buffers > 0 {
                let mut skip = vec![0; (skip_buffers) as usize * self.buffer_size];
                let bytes_skipped = self.reader.read(&mut skip[..])?;
                self.offset += bytes_skipped as u64;
            }
            self.read_next_buffer()?;
            Ok(offset_in_buffer)
        }
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
                // don't seek befor the end of the current buffer
                if pos < self.offset {
                    return Err(Error::new(ErrorKind::InvalidData, "cannot seek to discarded buffer"));
                }

                // We can seek behind the end of the current buffer,
                // but this requires discarding the current buffer
                // and reloading a new buffer.
                let mut position_in_buffer = pos - self.offset;
                if position_in_buffer as usize >= self.bytes_in_buffer {
                    position_in_buffer = self.seek_until_position(position_in_buffer)?;
                }
                Ok(self.cursor.seek(SeekFrom::Start(position_in_buffer))? + self.offset)
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