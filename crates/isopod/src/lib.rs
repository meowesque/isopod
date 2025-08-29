use std::borrow::Cow;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

#[repr(u8)]
enum VolumeDescriptorType {
  BootRecord = 0,
  PrimaryVolumeDescriptor = 1,
  SupplementaryVolumeDescriptor = 2,
  VolumePartitionDescriptor = 3,
  Other(u8),
  VolumeDescriptorSetTerminator = 255,
}

/// Derived from [OSDev ISO 9660](https://wiki.osdev.org/ISO_9660).
#[derive(Debug)]
pub struct PrimaryVolumeDescriptor {
  /// Always 0x01 for a Primary Volume Descriptor.
  type_code: u8,
  /// Standard identifier. (Always `CD001`).
  id: [u8; 5],
  /// Version. (Always `0x01`).
  version: u8,
  /// Unused. (Always `0x0`).
  unused1: u8,
  /// Name of the system that can act upon sectors `0x00` to `0x0F` for the volume.
  system_id: [u8; 32],
  /// Identification of this volume.
  volume_identifier: [u8; 32],
  unused2: u64,
  /// Number of Logical Blocks in which the volume is recorded.
  volume_space_size: u16,
  unused3: [u8; 32],
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
  /// TODO(meowesque): Comment appropriately
  directory_entry_identifier: [u8; 34],
  /// Identifier of the volume set of which this volume is a member.
  volume_set_identifier: [u8; 128],
  /// The volume publisher. For extended publisher information, the first byte should
  /// be `0x5F`, followed by the filename of a file in the root directory. If not
  /// specified, all bytes should be `0x20`.
  publisher_identifier: [u8; 128],
  /// The identifier of the person(s) who prepared the data for this volume. For
  /// extended preparation information, the first byte should be `0x5F`, followed
  /// by the filename of a file in the root directory. If not specified, all bytes
  /// should be `0x20`.
  data_preparer_identifier: [u8; 128],
  /// Identifies how the data is recorded on this volume. For extended information, the
  /// first byte should be `0x5F`, followed by the filename of a file in the root directory.
  /// If not specified all bytes should be `0x20`.
  application_identifier: [u8; 128],
  /// Filename of a file in the root diretory that contains copyright information for this volume
  /// set. If not specified, all bytes should be 0x20.
  copyright_file_identifier: [u8; 37],
  /// Filename of a file in the root directory that contains abstract information for this volum
  /// set. If not specified, all bytes should be `0x20`.
  abstract_file_identifier: [u8; 37],
  /// Filename of a file in the root directory that contains bibliographic information
  /// for this volume set. If not specified all bytes should be `0x20`.
  bibliographic_file_identifier: [u8; 37],
  /// The date and time of when the volume was created
  volume_creation_date: [u8; 17],
  /// The date and time after which this volume is considered to be obsolete. If not specified
  /// then the volume is never considered to be obsolete.
  volume_expiration_date: [u8; 17],
  /// The date and time after which the volume may be used. IF not specified, the volume may
  /// be used immediately.
  volume_effective_date: [u8; 17],
  /// The directory records and path table version (always `0x01`).
  file_structure_version: u8,
  unused4: u8,
  application_used: [u8; 512],
  reserved: [u8; 653],
}

#[derive(Debug)]
pub enum VolumeDescriptor {
  PrimaryVolumeDescriptor(PrimaryVolumeDescriptor),
}

impl VolumeDescriptor {
  fn parse(storage: impl AsRef<[u8]>) -> Result<Self> {
    todo!()
  }
}

/// The Path Table contains a well-ordered sequence of records describing every directory extent on the CD.
/// There are some exceptions with this: the Path Table can only contain 65536 records, due to the length of the `parent_directory_number` field.
struct PathTableEntry<'a> {
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
  directory_name: Cow<'a, [u8]>,
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

struct DirectoryRecord<'a> {
  directory_record_length: u8,
  extended_attribute_record_length: u8,
  /// LBA of location.
  extent_lba: u32,
  extent_size: u32,
  record_date: [u8; 7],
  flags: u8,
  interleaved_unit_size: u8,
  interleave_gap_size: u8,
  volume_sequence_number: u32,
  identifier_length: u8,
  identifier: Cow<'a, [u8]>,
  padding: u8,
  // TODO(meowesque): ISO 9660 extensions
}

// https://github.com/Adam-Vandervorst/PathMap/blob/master/src/arena_compact.rs#L625-L626
pub struct Iso<Storage> {
  storage: Storage,
}

impl<Storage> Iso<Storage> {
  pub fn new(storage: Storage) -> Self {
    Self { storage }
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
    const VOLUME_DESCRIPTOR_SIZE: usize = 2048;

    let storage = &self.storage.as_ref()[STARTING_SECTOR * 2048..];

    let mut position = 0;
    let mut volumes = Vec::new();

    while position < storage.len() || storage[position] != 255 /* TODO(meowesque): Unreadable, lazy */ {
      let descriptor_bytes = &storage[position..position + VOLUME_DESCRIPTOR_SIZE];
      let volume_descriptor = VolumeDescriptor::parse(descriptor_bytes)?;

      volumes.push(volume_descriptor);
      position += VOLUME_DESCRIPTOR_SIZE;
    }

    Ok(volumes)
  }
}

impl<Storage> Iso<Storage> where Storage: std::io::Write {}
