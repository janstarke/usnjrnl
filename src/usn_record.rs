#![allow(non_snake_case)]
use binread::derive_binread;
use binread::prelude::*;
use binread::ReadOptions;
use chrono::{DateTime, Utc};
use std::io::SeekFrom;
use std::io::{Read, Seek};
use winstructs::ntfs::mft_reference::MftReference;
use winstructs::timestamp::WinTimestamp;

use crate::usn_reader_error::*;
use crate::usn_reason::*;

#[derive(Debug)]
pub struct CommonUsnRecord {
    pub header: UsnRecordCommonHeader,
    pub data: UsnRecordData,
}

impl CommonUsnRecord {
    pub fn from<RS>(data: &mut RS) -> std::result::Result<Self, UsnReaderError>
    where
        RS: Read + Seek,
    {
        let mut header: UsnRecordCommonHeader = data.read_le()?;

        while header.RecordLength == 0 {
            /* looks like a cluster change, round index up to the next cluster */
            let current_position = data.stream_position()?;
            let diff = 0x1000 - (current_position & 0xfff);

            /* forward reader to new offset */
            Self::ignore_bytes(data, diff as usize)?;

            /* reread header at new address */
            header = data.read_le()?;
        }

        let usn_data = match header.MajorVersion {
            2 => UsnRecordData::V2(UsnRecordV2::from(data)?),
            3 => {
                return Err(UsnReaderError::SyntaxError("Version 3 records (ReFS only) are not supported yes".to_string()));
            }
            4 => {
                return Err(UsnReaderError::SyntaxError("Version 4 records (ReFS only) are not supported yes".to_string()));
            }
            version => {
                return Err(UsnReaderError::SyntaxError(format!(
                    "invalid value for MajorVersion: {}",
                    version
                )));
            }
        };

        /*
         * read padding bytes, until the beginning of the next record
         *  we do this to prevent seeking, which is bad for buffered readers
         */
        let reader_position = usn_data.ending_position();
        let next_record =
            header.RecordLength as u64 - (reader_position - header.StartingPosition.0);
        Self::ignore_bytes(data, next_record as usize)?;

        Ok(Self {
            header,
            data: usn_data,
        })
    }

    fn ignore_bytes<R: Read + Seek>(reader: &mut R, count: usize) -> std::result::Result<(), UsnReaderError> {
        let mut _buffer = vec![0; count as usize];
        let bytes = reader.read(&mut _buffer[..])?;
        if bytes != count {
            /* we've read less bytes than expected, why may happen
             * if we reached the end of the file
             */

            /* let's see if we are at the end of the file */
            let current_position = match reader.stream_position() {
              Ok(pos) => pos,
              Err(_) => return Err(UsnReaderError::NoMoreData)
            };

            let end_position = reader.seek(SeekFrom::End(0))?;

            if current_position == end_position {
                return Err(UsnReaderError::NoMoreData)
            }

            /* seek back to the original position */
            reader.seek(SeekFrom::Start(current_position))?;
        }
        assert_eq!(bytes, count as usize);
        //reader.seek(SeekFrom::Current(count as i64))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum UsnRecordData {
    V2(UsnRecordV2),
    //
    // this entry is not supported yet
    //V3(UsnRecordV3),

    // The user always receives one or more USN_RECORD_V4 records followed by one
    // USN_RECORD_V3 record.
    //
    // this entry is not supported yet
    /*
    V4(UsnRecordV4),
    */
}

impl UsnRecordData {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::V2(data) => &data.TimeStamp,
        }
    }

    pub fn filename(&self) -> &str {
        match self {
            Self::V2(data) => &data.FileName,
        }
    }

    pub fn reason(&self) -> &UsnReason {
        match self {
            Self::V2(data) => &data.Reason,
        }
    }

    pub fn ending_position(&self) -> u64 {
        match self {
            Self::V2(data) => data.EndingPosition,
        }
    }
}

#[derive(Debug)]
pub struct CurPos(pub u64);

impl BinRead for CurPos {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _ro: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        Ok(CurPos(reader.stream_position()?))
    }
}

#[derive_binread]
#[derive(Debug)]
#[br(little)]
pub struct UsnRecordCommonHeader {
    StartingPosition: CurPos,

    /// The total length of a record, in bytes.
    ///
    /// Because USN record is a variable size, the RecordLength member should be
    /// used when calculating the address of the next record in an output buffer,
    /// for example, a buffer that is returned from operations for the
    /// DeviceIoControl function that work with different USN record types.
    ///
    /// For USN_RECORD_V4, the size in bytes of any change journal record is at
    /// most the size of the structure, plus (NumberOfExtents-1) times size of the
    /// USN_RECORD_EXTENT.
    pub RecordLength: u32,

    /// The major version number of the change journal software for this record.
    ///
    /// For example, if the change journal software is version 4.0, the major version number is 4.
    ///
    /// | Value | Description |
    /// |-|----|
    /// |2|The structure is a USN_RECORD_V2 structure and the remainder of the structure should be parsed using that layout.|
    /// |3|The structure is a USN_RECORD_V3 structure and the remainder of the structure should be parsed using that layout.|
    /// |4|The structure is a USN_RECORD_V4 structure and the remainder of the structure should be parsed using that layout.|
    pub MajorVersion: u16,

    /// The minor version number of the change journal software for this record. For example, if the change journal software
    /// is version 4.0, the minor version number is zero.
    pub MinorVersion: u16,
}

/// Contains the information for an update sequence number (USN) common header
/// which is common through USN_RECORD_V2, USN_RECORD_V3 and USN_RECORD_V4.
///
/// https://docs.microsoft.com/de-de/windows/win32/api/winioctl/ns-winioctl-usn_record_common_header

#[derive_binread]
#[br(little)]
pub struct BinaryUsnRecordV2 {
    /// the following field is not really part of UsnRecordV2
    #[br(temp)]
    pub StartingPosition: CurPos,

    pub FileReferenceNumber: u64,
    pub ParentFileReferenceNumber: u64,
    pub Usn: i64,
    pub TimeStamp: [u8; 8],
    pub Reason: u32,
    pub SourceInfo: u32,

    /// The unique security identifier assigned to the file or directory
    /// associated with this record.
    pub SecurityId: u32,

    /// The attributes for the file or directory associated with this record, as
    /// returned by the GetFileAttributes function. Attributes of streams
    /// associated with the file or directory are excluded.
    pub FileAttributes: u32,

    /// The length of the name of the file or directory associated with this
    /// record, in bytes. The FileName member contains this name. Use this member
    /// to determine file name length, rather than depending on a trailing '\0'
    /// to delimit the file name in FileName.
    #[br(temp, little)]
    pub FileNameLength: u16,

    /// The offset of the FileName member from the beginning of the structure.
    pub FileNameOffset: u16,

    /// The name of the file or directory associated with this record in Unicode
    /// format. This file or directory name is of variable length.
    ///
    /// When working with FileName, do not count on the file name that contains
    /// a trailing '\0' delimiter, but instead determine the length of the file
    /// name by using FileNameLength.
    ///
    /// Do not perform any compile-time pointer arithmetic using FileName.
    /// Instead, make necessary calculations at run time by using the value of
    /// the FileNameOffset member. Doing so helps make your code compatible with
    /// any future versions of USN_RECORD_V2.
    #[br(offset=StartingPosition.0 + (FileNameOffset as u64), count=FileNameLength/2)]
    pub FileName: Vec<u16>,

    EndingPosition: CurPos,
}

#[derive(Debug)]
pub struct UsnRecordV2 {
    pub FileReferenceNumber: MftReference,
    pub ParentFileReferenceNumber: MftReference,
    pub Usn: i64,
    pub TimeStamp: DateTime<Utc>,
    pub Reason: UsnReason,
    pub SourceInfo: u32,
    pub SecurityId: u32,
    pub FileAttributes: u32,
    pub FileName: String,
    pub EndingPosition: u64,
}

impl UsnRecordV2 {
    fn from<RS>(data: &mut RS) -> std::result::Result<Self, UsnReaderError>
    where
        RS: Read + Seek,
    {
        let record: BinaryUsnRecordV2 = data.read_le()?;

        let filename = String::from_utf16_lossy(&record.FileName);

        let file_reference = MftReference::from(record.FileReferenceNumber);
        let parent_reference = MftReference::from(record.ParentFileReferenceNumber);
        let timestamp = WinTimestamp::new(&record.TimeStamp[..])
            .map_err(|_| UsnReaderError::FailedToReadWindowsTime(record.TimeStamp))?
            .to_datetime();
        Ok(Self {
            FileReferenceNumber: file_reference,
            ParentFileReferenceNumber: parent_reference,
            Usn: record.Usn,
            TimeStamp: timestamp,
            Reason: UsnReason::from(record.Reason),
            SourceInfo: record.SourceInfo,
            SecurityId: record.SecurityId,
            FileAttributes: record.FileAttributes,
            FileName: filename,
            EndingPosition: record.EndingPosition.0,
        })
    }
}

/*
#[derive(BinRead)]
#[br(little)]
pub struct UsnRecordV3 {
  pub FileReferenceNumber: [u8; 16],
  pub ParentFileReferenceNumber: [u8; 16],
  pub Usn: i64,
  pub TimeStamp: i64,
  #[packed_field(size_bytes="4")]
  pub Reason: u32,
  pub SourceInfo: u32,
  pub SecurityId: u32,
  pub FileAttributes: u32,
  pub FileNameLength: u16,
  pub FileNameOffset: u16,
  pub FileName: u16,
}
*/
/*
#[derive(BinRead)]
#[br(little)]
pub struct UsnRecordV4 {
  pub FileReferenceNumber: [u8; 16],
  pub ParentFileReferenceNumber: [u8; 16],
  pub Usn: i64,
  #[packed_field(size_bytes="4")]
  pub Reason: UsnReason,
  pub SourceInfo: u32,
  pub RemainingExtents: u32,
  pub NumberOfExtents: u16,
  pub ExtentSize: u16,
}

#[derive(BinRead)]
#[br(little)]
pub struct UsnRecordExtend {
  pub Offset: u64,
  pub Length: u64,
}
*/
