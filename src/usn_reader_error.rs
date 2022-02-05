use std::{fmt, io::ErrorKind};

#[derive(Debug)]
pub enum UsnReaderError {
    IO(std::io::Error),
    BinRead(binread::Error),
    SyntaxError(String),
    FailedToReadWindowsTime([u8;8]),
    NoMoreData,
  }
  
  impl From<std::io::Error> for UsnReaderError {
    fn from(err: std::io::Error) -> Self {
      Self::IO(err)
    }
  }

  impl From<binread::Error> for UsnReaderError {
    fn from(err: binread::Error) -> Self {
      match err {
        binread::Error::Io(ref io_error) => {
          if io_error.kind() == ErrorKind::UnexpectedEof {
            Self::NoMoreData
          } else {
            Self::BinRead(err)
          }
        }
        _ => Self::BinRead(err)
      }
    }
  }
  
  impl fmt::Display for UsnReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        Self::IO(io_error) => write!(f, "IO Error: {}", io_error),
        Self::BinRead(io_error)  => write!(f, "BinRead Error: {}", io_error),
        Self::FailedToReadWindowsTime(data) => write!(f, "failed to read windows time: {:?}", data),
        Self::SyntaxError(err) => write!(f, "Syntax Error: {}", err),
        Self::NoMoreData => write!(f, "no more data"),
      }
    }
  }