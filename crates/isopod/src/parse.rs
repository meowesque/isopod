use byteorder::ByteOrder;
pub use nom::bytes::*;
pub use nom::combinator::*;
pub use nom::number::complete::*;
pub use nom::sequence::*;
pub use nom::*;

use crate::spec;

fn take_string_n(i: &[u8], n: usize) -> IResult<&[u8], &str> {
  map(map_res(take(n), str::from_utf8), str::trim_end).parse(i)
}

fn take_utf16be_n(i: &[u8], n: usize) -> IResult<&[u8], String> {
  // TODO(meowesque): This is unoptimal

  let mut utf16 = Vec::new();

  for ix in 0..(n / 2) {
    utf16.push(byteorder::BigEndian::read_u16(&i[ix * 2..ix * 2 + 2]));
  }

  Ok((&i[n..], String::from_utf16_lossy(&utf16).trim().to_owned()))
}

fn lsb_msb_u16(i: &[u8]) -> IResult<&[u8], u16> {
  terminated(le_u16, take(2usize)).parse(i)
}

fn lsb_msb_u32(i: &[u8]) -> IResult<&[u8], u32> {
  terminated(le_u32, take(4usize)).parse(i)
}

fn ascii_i32(i: &[u8], n: usize) -> IResult<&[u8], i32> {
  map_res(map_res(take(n), str::from_utf8), str::parse::<i32>).parse(i)
}

pub(crate) trait Parse: Sized {
  type Output;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output>;
}

impl Parse for spec::IsoDateTime {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    let (i, years_since_1900) = le_u8(i)?;
    let (i, month) = le_u8(i)?;
    let (i, day) = le_u8(i)?;
    let (i, hour) = le_u8(i)?;
    let (i, minute) = le_u8(i)?;
    let (i, second) = le_u8(i)?;
    let (i, offset) = le_i8(i)?;

    Ok((
      i,
      Self {
        years_since_1900,
        month,
        day,
        hour,
        minute,
        second,
        offset,
      },
    ))
  }
}

impl Parse for spec::IsoPreciseDateTime {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    let (i, year) = ascii_i32(i, 4)?;
    let (i, month) = ascii_i32(i, 2)?;
    let (i, day) = ascii_i32(i, 2)?;
    let (i, hour) = ascii_i32(i, 2)?;
    let (i, minute) = ascii_i32(i, 2)?;
    let (i, second) = ascii_i32(i, 2)?;
    let (i, hundredths) = ascii_i32(i, 2)?;
    let (i, offset) = le_i8(i)?;

    Ok((
      i,
      Self {
        year: year as u16,
        month: month as u8,
        day: day as u8,
        hour: hour as u8,
        minute: minute as u8,
        second: second as u8,
        hundredths: hundredths as u8,
        offset,
      },
    ))
  }
}

impl Parse for spec::DirectoryRecord {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    // let i_oldlen = i.len();

    let (i, record_length) = le_u8(i)?;
    let (i, extended_attribute_record_length) = le_u8(i)?;
    let (i, extent_lba) = lsb_msb_u32(i)?;
    let (i, extent_length) = lsb_msb_u32(i)?;

    let (i, recording_date) = spec::IsoDateTime::parse(i)?;

    let (i, file_flags) = le_u8(i)?;

    let (i, file_unit_size) = le_u8(i)?;
    let (i, interleave_gap_size) = le_u8(i)?;

    let (i, volume_sequence_number) = lsb_msb_u16(i)?;

    let (i, identifier_length) = le_u8(i)?;
    let (i, identifier) = take_string_n(i, identifier_length as usize)?;

    Ok((
      i,
      Self {
        record_length,
        extended_attribute_record_length,
        extent_lba,
        extent_length,
        recording_date,
        file_flags: spec::FileFlags::from_bits_truncate(file_flags),
        file_unit_size,
        interleave_gap_size,
        volume_sequence_number,
        identifier_length,
        identifier: identifier.to_owned(),
      },
    ))
  }
}

impl Parse for spec::VolumeDescriptorIdentifier {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    map_opt(take(5usize), spec::VolumeDescriptorIdentifier::from_bytes).parse(i)
  }
}

impl Parse for spec::PrimaryVolumeDescriptor {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    let (i, _vd_type) = take(1usize).parse(i)?;
    let (i, standard_identifier) = spec::VolumeDescriptorIdentifier::parse(i)?;
    let (i, version) = le_u8(i)?;
    let (i, _unused1) = take(1usize).parse(i)?;
    let (i, system_identifier) = take_string_n(i, 32)?;

    let (i, volume_identifier) = take_string_n(i, 32)?;
    let (i, _unused2) = take(8usize).parse(i)?;
    let (i, volume_space_size) = lsb_msb_u32(i)?;
    let (i, _unused3) = take(32usize).parse(i)?;
    let (i, volume_set_size) = lsb_msb_u16(i)?;
    let (i, volume_sequence_number) = lsb_msb_u16(i)?;

    let (i, logical_block_size) = lsb_msb_u16(i)?;
    let (i, path_table_size) = lsb_msb_u32(i)?;

    let (i, type_l_path_table_lba) = le_u32(i)?;
    let (i, optional_type_l_path_table_lba) = le_u32(i)?;
    let (i, type_m_path_table_lba) = be_u32(i)?;
    let (i, optional_type_m_path_table_lba) = be_u32(i)?;

    let (i, root_directory_record) = spec::DirectoryRecord::parse(i)?;

    let (i, volume_set_identifier) = take_string_n(i, 128)?;
    let (i, publisher_identifier) = take_string_n(i, 128)?;
    let (i, data_preparer_identifier) = take_string_n(i, 128)?;
    let (i, application_identifier) = take_string_n(i, 128)?;
    let (i, copyright_file_identifier) = take_string_n(i, 38)?;
    let (i, abstract_file_identifier) = take(36usize).parse(i)?;
    let (i, bibliographic_file_identifier) = take(37usize).parse(i)?;

    let (i, volume_creation_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_modification_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_expiration_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_effective_date) = spec::IsoPreciseDateTime::parse(i)?;

    let (i, file_structure_version) = le_u8(i)?;

    let (i, _unused4) = take(1usize).parse(i)?;

    let (i, application_data) = take(512usize).parse(i)?;

    // TODO(meowesque): Implement parsing for ISO 9660 extensions.
    let (i, reserved) = take(653usize).parse(i)?;

    Ok((
      i,
      Self {
        standard_identifier,
        version,
        system_identifier: system_identifier.to_owned(),
        volume_identifier: volume_identifier.to_owned(),
        volume_space_size,
        volume_set_size,
        volume_sequence_number,
        logical_block_size,
        path_table_size,
        type_l_path_table_lba,
        optional_type_l_path_table_lba,
        type_m_path_table_lba,
        optional_type_m_path_table_lba,
        root_directory_record,
        volume_set_identifier: volume_set_identifier.to_owned(),
        publisher_identifier: publisher_identifier.to_owned(),
        data_preparer_identifier: data_preparer_identifier.to_owned(),
        application_identifier: application_identifier.to_owned(),
        copyright_file_identifier: copyright_file_identifier.to_owned(),
        abstract_file_identifier: abstract_file_identifier.try_into().unwrap(),
        bibliographic_file_identifier: bibliographic_file_identifier.try_into().unwrap(),
        volume_creation_date,
        volume_modification_date,
        volume_expiration_date,
        volume_effective_date,
        file_structure_version,
        application_data: application_data.try_into().unwrap(),
        reserved: reserved.try_into().unwrap(),
      },
    ))
  }
}

impl Parse for spec::SupplementaryVolumeDescriptor {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    let (i, _vd_type) = take(1usize).parse(i)?;
    let (i, standard_identifier) = spec::VolumeDescriptorIdentifier::parse(i)?;
    let (i, version) = le_u8(i)?;
    let (i, volume_flags) = le_u8(i)?;
    let (i, system_id) = take_utf16be_n(i, 32)?;

    let (i, volume_id) = take_utf16be_n(i, 32)?;
    let (i, _unused1) = take(8usize).parse(i)?;
    let (i, volume_space_size) = lsb_msb_u32(i)?;
    let (i, escape_sequences) = take(32usize).parse(i)?;
    let (i, volume_set_size) = lsb_msb_u16(i)?;
    let (i, volume_sequence_number) = lsb_msb_u16(i)?;

    let (i, logical_block_size) = lsb_msb_u16(i)?;
    let (i, path_table_size) = lsb_msb_u32(i)?;

    let (i, type_l_path_table_lba) = le_u32(i)?;
    let (i, optional_type_l_path_table_lba) = le_u32(i)?;
    let (i, type_m_path_table_lba) = be_u32(i)?;
    let (i, optional_type_m_path_table_lba) = be_u32(i)?;

    let (i, root_directory_record) = spec::DirectoryRecord::parse(i)?;

    let (i, volume_set_identifier) = take_utf16be_n(i, 128)?;
    let (i, publisher_identifier) = take_utf16be_n(i, 128)?;
    let (i, data_preparer_identifier) = take_utf16be_n(i, 128)?;
    let (i, application_identifier) = take_utf16be_n(i, 128)?;
    let (i, copyright_file_identifier) = take_utf16be_n(i, 38)?;
    let (i, abstract_file_identifier) = take(36usize).parse(i)?;
    let (i, bibliographic_file_identifier) = take(37usize).parse(i)?;

    let (i, volume_creation_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_modification_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_expiration_date) = spec::IsoPreciseDateTime::parse(i)?;
    let (i, volume_effective_date) = spec::IsoPreciseDateTime::parse(i)?;

    let (i, file_structure_version) = le_u8(i)?;

    let (i, _unused4) = take(1usize).parse(i)?;

    let (i, application_data) = take(512usize).parse(i)?;

    // TODO(meowesque): Implement parsing for ISO 9660 extensions.
    let (i, reserved) = take(653usize).parse(i)?;

    Ok((
      i,
      Self {
        standard_identifier,
        version,
        volume_flags: spec::SupplementaryVolumeFlags::from_bits_truncate(volume_flags),
        system_identifier: system_id.to_string(),
        volume_identifier: volume_id.to_string(),
        volume_space_size,
        escape_sequences: escape_sequences.try_into().unwrap(),
        volume_set_size,
        volume_sequence_number,
        logical_block_size,
        path_table_size,
        type_l_path_table_lba,
        optional_type_l_path_table_lba,
        type_m_path_table_lba,
        optional_type_m_path_table_lba,
        root_directory_record,
        volume_set_identifier: volume_set_identifier.to_owned(),
        publisher_identifier: publisher_identifier.to_owned(),
        data_preparer_identifier: data_preparer_identifier.to_owned(),
        application_identifier: application_identifier.to_owned(),
        copyright_file_identifier: copyright_file_identifier.to_owned(),
        abstract_file_identifier: abstract_file_identifier.try_into().unwrap(),
        bibliographic_file_identifier: bibliographic_file_identifier.try_into().unwrap(),
        volume_creation_date,
        volume_modification_date,
        volume_expiration_date,
        volume_effective_date,
        file_structure_version,
        application_data: application_data.try_into().unwrap(),
        reserved: reserved.try_into().unwrap(),
      },
    ))
  }
}

impl Parse for spec::VolumeDescriptor {
  type Output = Option<Self>;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    // The first byte is the volume descriptor type
    match spec::VolumeDescriptorType::from_u8(i[0]) {
      spec::VolumeDescriptorType::PrimaryVolumeDescriptor => {
        let (i, vd) = spec::PrimaryVolumeDescriptor::parse(i)?;
        Ok((i, Some(spec::VolumeDescriptor::Primary(vd))))
      }
      spec::VolumeDescriptorType::SupplementaryVolumeDescriptor => {
        let (i, vd) = spec::SupplementaryVolumeDescriptor::parse(i)?;
        Ok((i, Some(spec::VolumeDescriptor::Supplementary(vd))))
      }
      spec::VolumeDescriptorType::BootRecord => {
        // TODO(meowesque): Handle different boot system types
        let (i, vd) = spec::BootRecordVolumeDescriptor::parse(i)?;
        Ok((i, Some(spec::VolumeDescriptor::Boot(vd))))
      }
      spec::VolumeDescriptorType::VolumeDescriptorSetTerminator => Ok((i, None)),
      _ => unimplemented!(),
    }
  }
}

impl Parse for spec::BootRecordVolumeDescriptor {
  type Output = Self;

  fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
    let i_oldlen = i.len();

    let (i, _vd_type) = take(1usize).parse(i)?;
    let (i, standard_identifier) = spec::VolumeDescriptorIdentifier::parse(i)?;
    let (i, version) = le_u8(i)?;
    let (i, boot_system_identifier) = take_string_n(i, 32)?;
    let (i, boot_identifier) = take_string_n(i, 32)?;
    let (i, absolute_pointer) = le_u32(i)?;
    let (i, _boot_system_use) = take(1973usize).parse(i)?;

    Ok((
      i,
      Self {
        standard_identifier,
        version,
        boot_system_identifier: boot_system_identifier.to_string(),
        boot_identifier: boot_identifier.to_string(),
        absolute_pointer,
      },
    ))
  }
}

mod el_torito {
  use super::*;

  impl Parse for spec::el_torito::InitialSectionHeaderEntry {
    type Output = Self;

    fn parse(i: &[u8]) -> IResult<&[u8], Self::Output> {
      let (i, header_id) = le_u8(i)?;
      let (i, platform_id) = map_opt(le_u8, spec::el_torito::PlatformId::from_u8).parse(i)?;
      let (i, identifier) = take_string_n(i, 24)?;
      let (i, checksum) = le_u16(i)?;
      let (i, bootable) = map_opt(le_u8, spec::el_torito::BootIndicator::from_u8).parse(i)?;
      let (i, _unused1) = take(1usize).parse(i)?;

      todo!()
    }
  }
}
