use std::fmt;
use std::io::{SeekFrom, BufReader, BufRead, Seek, Read};
use std::path::PathBuf;
use std::collections::HashMap;
use std::str;

use num_traits::FromPrimitive;

use crate::errors::Error;
use crate::parsers::ptu::{PTUTag, PTUTagType, FILE_TAG_END, Header};

impl fmt::Display for PTUTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PTUTag::Empty8 => write!(f, ""),
            PTUTag::Bool8(x) => write!(f, "{}", x),
            PTUTag::Int8(x) => write!(f, "{}", x),
            PTUTag::BitSet64(x) => write!(f, "{}", x),
            PTUTag::Color8(x) => write!(f, "{}", x),
            PTUTag::Float8(x) => write!(f, "{}", x),
            PTUTag::TDateTime(x) => write!(f, "{}", x),
            PTUTag::Float8Array(x) => write!(f, "{:?}", x),
            PTUTag::AnsiString8(x) => write!(f, "{}", x),
            PTUTag::WideString(x) => write!(f, "{}", x),
            PTUTag::BinaryBlob(x) => write!(f, "{:?}", x),
        }
    }
}


fn read_string(slice: &[u8], size: usize) -> Option<String> {
    assert!(2 * size <= slice.len());
    let iter = (0..size).map(|i| u16::from_be_bytes([slice[2 * i], slice[2 * i + 1]]));

    std::char::decode_utf16(iter)
        .collect::<Result<String, _>>()
        .ok()
}

pub(in super) fn read_ptu_header(filename: &PathBuf) -> Result<Header, Error> {
    let offset = 16;
    let mut buffered = BufReader::new(std::fs::File::open(filename)?);
    let mut header = HashMap::new();

    buffered.seek(SeekFrom::Start(offset))?;
    let mut tagname_buffer: [u8; 32] = [0; 32];
    let mut index_buffer: [u8; 4] = [0; 4];
    let mut type_buffer: [u8; 4] = [0; 4];
    let mut value_buffer: [u8; 8] = [0; 8];

    loop {
        buffered.read_exact(&mut tagname_buffer)?;
        buffered.read_exact(&mut index_buffer)?;
        buffered.read_exact(&mut type_buffer)?;
        buffered.read_exact(&mut value_buffer)?;

        let (tag_name, _tag_idx, tag_type) = read_tag(tagname_buffer, index_buffer, type_buffer)?;

        if tag_name == *FILE_TAG_END {
            break;
        }

        let tag = process_tag(tag_type, value_buffer, &mut buffered)?;
        header.insert(tag_name, tag);
    }

    let current_pos = buffered.seek(SeekFrom::Current(0))?;
    header.insert(String::from("DataOffset"), PTUTag::Int8(current_pos as i64));
    Ok(header)
}

fn process_tag<R: BufRead>(
    tag_type: PTUTagType,
    value_buffer: [u8; 8],
    buffered: &mut R,
) -> Result<PTUTag, Error> {
    let tag = match tag_type {
        PTUTagType::Empty8 => PTUTag::Empty8,
        PTUTagType::Bool8 => {
            let bool_u64 = i64::from_le_bytes(value_buffer);
            PTUTag::Bool8(bool_u64 != 0)
        }
        PTUTagType::Int8 => PTUTag::Int8(i64::from_le_bytes(value_buffer)),
        PTUTagType::BitSet64 => {
            let bitset = i64::from_le_bytes(value_buffer);
            PTUTag::BitSet64(bitset)
        }
        PTUTagType::Color8 => {
            let color = i64::from_le_bytes(value_buffer);
            PTUTag::Color8(color)
        }
        PTUTagType::Float8 => {
            let float = f64::from_le_bytes(value_buffer);
            PTUTag::Float8(float)
        }
        PTUTagType::TDateTime => {
            let dtime_double = u64::from_le_bytes(value_buffer) as f64;
            let epoch_diff: f64 = 25569.;
            let secs_in_day: f64 = 86400.;
            let epoch_time = (dtime_double - epoch_diff) * secs_in_day;
            PTUTag::TDateTime(epoch_time) // Unix time
        }
        PTUTagType::Float8Array => {
            let n_bytes_array = u64::from_le_bytes(value_buffer);
            let float_count = n_bytes_array / 8;
            let mut float_array: Vec<f64> = Vec::with_capacity(float_count as usize);
            let mut float_buffer: [u8; 8] = [0; 8];
            for _ in 0..float_count {
                buffered.read_exact(&mut float_buffer)?;
                let next_float = f64::from_le_bytes(float_buffer);
                float_array.push(next_float);
            }
            PTUTag::Float8Array(float_array)
        }
        PTUTagType::WideString => {
            let string_length = u64::from_le_bytes(value_buffer) as usize;
            let mut string_buffer: Vec<u8> = vec![0; string_length];
            buffered.read_exact(&mut string_buffer)?;
            let wide_string = read_string(&string_buffer, string_length).unwrap();
            PTUTag::WideString(wide_string.trim_matches(char::from(0)).to_string())
        }
        PTUTagType::BinaryBlob => {
            let n_bytes_blob = u64::from_le_bytes(value_buffer);
            let mut blob_buffer: Vec<u8> = vec![0; n_bytes_blob as usize];
            buffered.read_exact(&mut blob_buffer)?;
            PTUTag::BinaryBlob(blob_buffer)
        }
        PTUTagType::AnsiString8 => {
            let string_length = u64::from_le_bytes(value_buffer);
            let mut string_buffer: Vec<u8> = vec![0; string_length as usize];
            buffered.read_exact(&mut string_buffer)?;
            let value = str::from_utf8(&string_buffer)
                .ok()
                .ok_or_else(|| Error::InvalidHeader(String::from(
                    "Invalid utf8 string in header.",
                )))?;
            PTUTag::AnsiString8(value.to_string().trim_matches(char::from(0)).to_string())
        }
    };
    Ok(tag)
}

fn read_tag(
    tagname_buffer: [u8; 32],
    index_buffer: [u8; 4],
    type_buffer: [u8; 4],
) -> Result<(String, i32, PTUTagType), Error> {
    let tag_index = i32::from_le_bytes(index_buffer);

    let tag_name = str::from_utf8(&tagname_buffer)
        .ok()
        .ok_or_else(|| Error::InvalidHeader(String::from(
            "Invalid utf8 string in header.",
        )))?
        .trim_matches(char::from(0));
    let tag_name = if tag_index > -1 {
        format!("{}{}", tag_name, tag_index)
    } else {
        tag_name.to_string()
    };

    let tag_type = FromPrimitive::from_u32(u32::from_le_bytes(type_buffer))
        .ok_or_else(|| Error::InvalidHeader(String::from("Invalid PTUTag type")))?;

    Ok((tag_name, tag_index, tag_type))
}
