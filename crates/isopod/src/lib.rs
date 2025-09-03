mod parse;
mod udf;

use parse::*;
use std::borrow::Cow;

// TODO(meowesque): Allow this to be configurable
const SECTOR_SIZE: usize = 2048;
const VOLUME_DESCRIPTOR_SIZE: usize = 2048;
const VOLUME_DESCRIPTOR_IDENTIFIER_SIZE: usize = 5;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

#[derive(Debug)]
struct IsoDateTime {
  years_since_1900: u8,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  /// Offset from GMT in 15 minute intervals from -48 (West) to +52 (East).
  offset: i8,
}

impl IsoDateTime {
  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
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

#[derive(Debug)]
struct IsoPreciseDateTime {
  year: i32,
  month: i32,
  day: i32,
  hour: i32,
  minute: i32,
  second: i32,
  hundreths: i32,
  offset: i8,
}

impl IsoPreciseDateTime {
  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
    let (i, year) = ascii_i32(i, 4)?;
    let (i, month) = ascii_i32(i, 2)?;
    let (i, day) = ascii_i32(i, 2)?;
    let (i, hour) = ascii_i32(i, 2)?;
    let (i, minute) = ascii_i32(i, 2)?;
    let (i, second) = ascii_i32(i, 2)?;
    let (i, hundreths) = ascii_i32(i, 2)?;
    let (i, offset) = le_i8(i)?;

    Ok((
      i,
      Self {
        year,
        month,
        day,
        hour,
        minute,
        second,
        hundreths,
        offset,
      },
    ))
  }
}

#[repr(u8)]
#[derive(Debug)]
enum VolumeDescriptorType {
  BootRecord = 0,
  PrimaryVolumeDescriptor = 1,
  SupplementaryVolumeDescriptor = 2,
  VolumePartitionDescriptor = 3,
  Other(u8),
  VolumeDescriptorSetTerminator = 255,
}

impl VolumeDescriptorType {
  fn from_u8(value: u8) -> Self {
    match value {
      0 => Self::BootRecord,
      1 => Self::PrimaryVolumeDescriptor,
      2 => Self::SupplementaryVolumeDescriptor,
      3 => Self::VolumePartitionDescriptor,
      255 => Self::VolumeDescriptorSetTerminator,
      _ => Self::Other(value),
    }
  }
}

#[derive(Debug)]
enum VolumeDescriptorIdentifier {
  /// ISO 9660 file system.
  Cd001,
  /// Extended descriptor section.
  Bea01,
  /// URF filesystem.
  Nsr02,
  /// UDF filesystem.
  Nsr03,
  /// Boot loader location and entry point address.
  Boot2,
  /// Denotes the end of the extended descriptor section.
  Tea01,
}

impl VolumeDescriptorIdentifier {
  fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> {
    Some(match bytes.as_ref() {
      b"CD001" => Self::Cd001,
      b"BEA01" => Self::Bea01,
      b"NSR02" => Self::Nsr02,
      b"NSR03" => Self::Nsr03,
      b"BOOT2" => Self::Boot2,
      b"TEA01" => Self::Tea01,
      _ => return None,
    })
  }

  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
    let (i, bytes) = take(5usize).parse(i)?;

    match Self::from_bytes(bytes) {
      Some(id) => Ok((i, id)),
      None => todo!(),
    }
  }
}

#[repr(u16)]
#[derive(Debug)]
enum DescriptorTagIdentifier {
  PrimaryVolumeDescriptor = 0x0001,
  AnchorVolumeDescriptorPointer = 0x0002,
  VolumeDescriptorPointer = 0x0003,
  ImplementationUseVolumeDescriptor = 0x0004,
  PartitionDescriptor = 0x0005,
  LogicalVolumeDescriptor = 0x0006,
  UnallocatedSpaceDescriptor = 0x0007,
  TerminatingDescriptor = 0x0008,
  LogicalVolumeIntegrityDescriptor = 0x0009,
  FileSetDescriptor = 0x0100,
  FileIdentifierDescriptor = 0x0101,
  AllocationExtentDescriptor = 0x0102,
  IndirectEntry = 0x0103,
  TerminalEntry = 0x0104,
  FileEntry = 0x0105,
  ExtendedAttributeHeaderDescriptor = 0x0106,
  UnallocatedSpaceEntry = 0x0107,
  SpaceBitmapDescriptor = 0x0108,
  PartitionIntegrityEntry = 0x0109,
  ExtendedFileEntry = 0x010A,
  Other(u16),
}

#[derive(Debug)]
struct DescriptorTag {
  tag_identifier: DescriptorTagIdentifier,
  descriptor_version: u16,
  tag_checksum: u8,
  reserved: u8,
  tag_serial_number: u16,
  descriptor_crc: u16,
  descriptor_crc_length: u16,
  tag_location: u32,
}

impl DescriptorTag {}

#[derive(Debug)]
struct Extent {
  /// Length in bytes of data pointed to.
  length: u32,
  /// Sector index of the data, relative to the start of the beginning of the volume.
  location: u32,
}

#[derive(Debug)]
struct AnchorVolumeDescriptor {
  descriptor_tag: DescriptorTag,
  main_volume_descriptor_sequence_extent: Extent,
  reserve_volume_descriptor_sequence_extent: Extent,
}

/// Derived from [OSDev ISO 9660](https://wiki.osdev.org/ISO_9660).
#[derive(Debug)]
pub struct PrimaryVolumeDescriptor {
  /// Standard identifier.
  id: VolumeDescriptorIdentifier,
  /// Version. (Always `0x01`).
  version: u8,
  /// Name of the system that can act upon sectors `0x00` to `0x0F` for the volume.
  system_id: String,
  /// Identification of this volume.
  volume_identifier: String,
  /// Number of Logical Blocks in which the volume is recorded.
  volume_space_size: u32,
  /// The size of the set in this logical volume (number of disks).
  volume_set_size: u16,
  /// The number of this disk in the Volume Set.
  volume_sequence_number: u16,
  /// The size in bytes of a logical block. NB: This means that
  /// a logical block on a CD could be something other than 2 KiB!
  logical_block_size: u16,
  /// The size in bytes of the path table.
  path_table_size: u32,
  /// Location of Type-L Path Table. The path table to contains only little-endian values.
  type_l_path_table_lba: u32,
  /// LBA location of the optional path table. The path table
  /// pointed to contains only little-endian values. Zero means
  /// that no optional path table exists.
  optional_type_l_path_table_lba: u32,
  /// LBA location of the path table. The path tbale pointed to contains
  /// only big-endian values.
  type_m_path_table_lba: u32,
  /// LBA location of the optional path table. The path table pointed
  /// to contains only big-endian values. Zero means that no optional path table exists.
  optional_type_m_path_table_lba: u32,
  root_directory_entry: DirectoryRecord,
  /// Identifier of the volume set of which this volume is a member.
  volume_set_identifier: String,
  /// The volume publisher. For extended publisher information, the first byte should
  /// be `0x5F`, followed by the filename of a file in the root directory. If not
  /// specified, all bytes should be `0x20`.
  publisher_identifier: String,
  /// The identifier of the person(s) who prepared the data for this volume. For
  /// extended preparation information, the first byte should be `0x5F`, followed
  /// by the filename of a file in the root directory. If not specified, all bytes
  /// should be `0x20`.
  data_preparer_identifier: String,
  /// Identifies how the data is recorded on this volume. For extended information, the
  /// first byte should be `0x5F`, followed by the filename of a file in the root directory.
  /// If not specified all bytes should be `0x20`.
  application_identifier: String,
  /// Filename of a file in the root diretory that contains copyright information for this volume
  /// set. If not specified, all bytes should be 0x20.
  copyright_file_identifier: String,
  /// Filename of a file in the root directory that contains abstract information for this volum
  /// set. If not specified, all bytes should be `0x20`.
  abstract_file_identifier: [u8; 36],
  /// Filename of a file in the root directory that contains bibliographic information
  /// for this volume set. If not specified all bytes should be `0x20`.
  bibliographic_file_identifier: [u8; 37],
  /// The date and time of when the volume was created
  volume_creation_date: IsoPreciseDateTime,
  /// The date and time of when the volume was last modified.
  volume_modification_date: IsoPreciseDateTime,
  /// The date and time after which this volume is considered to be obsolete. If not specified
  /// then the volume is never considered to be obsolete.
  volume_expiration_date: IsoPreciseDateTime,
  /// The date and time after which the volume may be used. IF not specified, the volume may
  /// be used immediately.
  volume_effective_date: IsoPreciseDateTime,
  /// The directory records and path table version (always `0x01`).
  file_structure_version: u8,
  application_used: [u8; 512],
  reserved: [u8; 653],
}

impl PrimaryVolumeDescriptor {
  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
    let (i, _vd_type) = take(1usize).parse(i)?;
    let (i, id) = VolumeDescriptorIdentifier::parse(i)?;
    let (i, version) = le_u8(i)?;
    let (i, _unused1) = take(1usize).parse(i)?;
    let (i, system_id) = take_string_n(i, 32)?;

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

    let (i, root_directory_entry) = DirectoryRecord::parse(i)?;

    let (i, volume_set_identifier) = take_string_n(i, 128)?;
    let (i, publisher_identifier) = take_string_n(i, 128)?;
    let (i, data_preparer_identifier) = take_string_n(i, 128)?;
    let (i, application_identifier) = take_string_n(i, 128)?;
    let (i, copyright_file_identifier) = take_string_n(i, 38)?;
    let (i, abstract_file_identifier) = take(36usize).parse(i)?;
    let (i, bibliographic_file_identifier) = take(37usize).parse(i)?;

    let (i, volume_creation_date) = IsoPreciseDateTime::parse(i)?;
    let (i, volume_modification_date) = IsoPreciseDateTime::parse(i)?;
    let (i, volume_expiration_date) = IsoPreciseDateTime::parse(i)?;
    let (i, volume_effective_date) = IsoPreciseDateTime::parse(i)?;

    let (i, file_structure_version) = le_u8(i)?;

    let (i, _unused4) = take(1usize).parse(i)?;

    let (i, application_used) = take(512usize).parse(i)?;

    // TODO(meowesque): Implement parsing for ISO 9660 extensions.
    let (i, reserved) = take(653usize).parse(i)?;

    Ok((
      i,
      Self {
        id,
        version,
        system_id: system_id.to_owned(),
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
        root_directory_entry,
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
        application_used: application_used.try_into().unwrap(),
        reserved: reserved.try_into().unwrap(),
      },
    ))
  }
}

struct LogicalVolumeDescriptor {
  descriptor_tag: DescriptorTag,
  volume_sequence_number: u32,
  // TODO(meowesque): Finish
  descriptor_character_set: [u8; 64],
  logical_volume_identifier: [u8; 128],
  logical_block_size: u32,
  domain_identifier: [u8; 32],
  logical_volume_contents_use: [u8; 16],
  map_table_length: u32,
  number_of_partition_maps: u16,
  implementation_identifier: [u8; 32],
  implementation_use: [u8; 128],
  integrity_sequence_extent: Extent,
}

#[derive(Debug)]
pub enum VolumeDescriptor {
  PrimaryVolumeDescriptor(PrimaryVolumeDescriptor),
}

impl VolumeDescriptor {
  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
    let i = &i.as_ref()[..VOLUME_DESCRIPTOR_SIZE];
    // The type is the first byte of any volume descriptor
    let vd_type = VolumeDescriptorType::from_u8(i[0]);

    match vd_type {
      VolumeDescriptorType::PrimaryVolumeDescriptor => {
        let (i, pvd) = PrimaryVolumeDescriptor::parse(i)?;

        Ok((i, Self::PrimaryVolumeDescriptor(pvd)))
      }
      _ => todo!(),
    }
  }
}

/// The Path Table contains a well-ordered sequence of records describing every directory extent on the CD.
/// There are some exceptions with this: the Path Table can only contain 65536 records, due to the length of the `parent_directory_number` field.
struct PathTableRecord {
  /// Length of Directory Identifier.
  directory_identifier_length: u8,
  /// Extended Attribute Record Length.
  extended_attribute_record_length: u8,
  /// Location of Extent (LBA). This is in a different format depending on
  /// whether this is the L-TAble or M-Table.
  extent_lba: u32,
  /// Directory number of parent directory (an index in to the path table).
  /// This is the field that limits the table to `65536` records.
  parent_directory_number: u16,
  directory_name: String,
}

/// Date time used within `DirectoryRecord`.
struct DirectoryRecordDateTime {
  years_since_1900: u8,
  month: u8,
  day: u8,
  hour: u8,
  minute: u8,
  second: u8,
  /// Offset from GMT in 15 minute intervals from `-48` (West) to `+52` (East).
  gmt_offset: i8,
}

#[derive(Debug)]
struct DirectoryRecord {
  length: u8,
  extended_attribute_record_length: u8,
  extent_lba: u32,
  extent_length: u32,

  record_date: IsoDateTime,

  file_flags: u8,

  interleaved_unit_size: u8,
  interleave_gap_size: u8,

  volume_sequence_number: u16,

  identifier_length: u8,
  identifier: String,
  // TODO(meowesque): ISO 9660 extensions
}

impl DirectoryRecord {
  fn parse(i: &[u8]) -> IResult<&[u8], Self> {
    let (i, length) = le_u8(i)?;
    let (i, extended_attribute_record_length) = le_u8(i)?;
    let (i, extent_lba) = lsb_msb_u32(i)?;
    let (i, extent_length) = lsb_msb_u32(i)?;

    let (i, record_date) = IsoDateTime::parse(i)?;

    let (i, file_flags) = le_u8(i)?;

    let (i, interleaved_unit_size) = le_u8(i)?;
    let (i, interleave_gap_size) = le_u8(i)?;

    let (i, volume_sequence_number) = lsb_msb_u16(i)?;

    let (i, identifier_length) = le_u8(i)?;
    let (i, identifier) = take_string_n(i, identifier_length as usize)?;

    Ok((
      i,
      Self {
        length,
        extended_attribute_record_length,
        extent_lba,
        extent_length,
        record_date,
        file_flags,
        interleaved_unit_size,
        interleave_gap_size,
        volume_sequence_number,
        identifier_length,
        identifier: identifier.to_owned(),
      },
    ))
  }
}

#[derive(Debug)]
pub struct Options {
  pub sector_size: u16,
}

impl Default for Options {
  fn default() -> Self {
    Self { sector_size: 2048 }
  }
}

// https://github.com/Adam-Vandervorst/PathMap/blob/master/src/arena_compact.rs#L625-L626
pub struct Iso<Storage> {
  options: Options,
  storage: Storage,
}

impl<Storage> Iso<Storage> {
  pub fn new(options: Options, storage: Storage) -> Self {
    Self { options, storage }
  }
}

impl<Storage> Iso<Storage>
where
  Storage: AsRef<[u8]>,
{
  pub fn storage_ref<'a>(&'a self) -> impl AsRef<[u8]> + 'a {
    self.storage.as_ref()
  }

  pub fn scan_volumes(&self) -> Result<Vec<VolumeDescriptor>> {
    const STARTING_SECTOR: usize = 0x10;

    let storage = &self.storage.as_ref()[STARTING_SECTOR * self.options.sector_size as usize..];

    let mut position = 0;
    let mut volumes = Vec::new();

    while position < storage.len() || storage[position] != 255
    /* TODO(meowesque): Unreadable, lazy */
    {
      let descriptor_bytes = &storage[position..position + VOLUME_DESCRIPTOR_SIZE];
      let (_, volume_descriptor) = VolumeDescriptor::parse(descriptor_bytes).expect("Something");

      volumes.push(volume_descriptor);
      position += VOLUME_DESCRIPTOR_SIZE;
    }

    Ok(volumes)
  }
}

impl<Storage> Iso<Storage> where Storage: std::io::Write {}
