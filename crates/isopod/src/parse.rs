use byteorder::ByteOrder;
pub use nom::bytes::*;
pub use nom::combinator::*;
pub use nom::number::complete::*;
pub use nom::sequence::*;
pub use nom::*;

pub(crate) fn take_string_n(i: &[u8], n: usize) -> IResult<&[u8], &str> {
  map(map_res(take(n), str::from_utf8), str::trim_end).parse(i)
}

pub(crate) fn take_utf16be_n(i: &[u8], n: usize) -> IResult<&[u8], String> {
  // TODO(meowesque): This is unoptimal
  
  let mut utf16 = Vec::new();
  
  for ix in 0..(n / 2) {
    utf16.push(byteorder::BigEndian::read_u16(&i[ix*2..ix*2+2]));
  }

  Ok((&i[n..], String::from_utf16_lossy(&utf16).trim().to_owned()))
}

pub(crate) fn lsb_msb_u16(i: &[u8]) -> IResult<&[u8], u16> {
  terminated(le_u16, take(2usize)).parse(i)
}

pub(crate) fn lsb_msb_u32(i: &[u8]) -> IResult<&[u8], u32> {
  terminated(le_u32, take(4usize)).parse(i)
}

pub(crate) fn ascii_i32(i: &[u8], n: usize) -> IResult<&[u8], i32> {
  map_res(map_res(take(n), str::from_utf8), str::parse::<i32>).parse(i)
}
