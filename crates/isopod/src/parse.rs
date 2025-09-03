pub use nom::bytes::*;
pub use nom::combinator::*;
pub use nom::number::complete::*;
pub use nom::sequence::*;
pub use nom::*;

pub(crate) fn take_string_n(i: &[u8], n: usize) -> IResult<&[u8], &str> {
  map(map_res(take(n), str::from_utf8), str::trim_end).parse(i)
}

pub(crate) fn lsb_msb_u16(i: &[u8]) -> IResult<&[u8], u16> {
  terminated(le_u16, take(2usize)).parse(i)
}

pub(crate) fn lsb_msb_u32(i: &[u8]) -> IResult<&[u8], u32> {
  terminated(le_u32, take(4usize)).parse(i)
}
