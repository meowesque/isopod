//! Low-level ISO 9660 (& Joliet) filesystem specification structures and constants.

pub const STARTING_SECTOR: u64 = 0x10;

/// Recording date utilized within directory records.
#[derive(Debug, Clone)]
pub struct IsoPreciseDateTime {
  pub year: u16,
  pub month: u8,
  pub day: u8,
  pub hour: u8,
  pub minute: u8,
  pub second: u8,
  pub hundredths: u8,
  pub offset: i8,
}

/// Date/Time format utilized in creation and modification dates.
#[derive(Debug, Clone)]
pub struct IsoDateTime {
  pub years_since_1900: u8,
  pub month: u8,
  pub day: u8,
  pub hour: u8,
  pub minute: u8,
  pub second: u8,
  /// Offset from GMT in 16 minute intervals from -48 (west) to +52 (east).
  pub offset: i8,
}

#[repr(u8)]
#[derive(Debug)]
pub enum VolumeDescriptorType {
  BootRecord = 0,
  PrimaryVolumeDescriptor = 1,
  SupplementaryVolumeDescriptor = 2,
  VolumePartitionDescriptor = 3,
  Other(u8),
  VolumeDescriptorSetTerminator = 255,
}

impl VolumeDescriptorType {
  pub(crate) fn from_u8(value: u8) -> Self {
    match value {
      0 => VolumeDescriptorType::BootRecord,
      1 => VolumeDescriptorType::PrimaryVolumeDescriptor,
      2 => VolumeDescriptorType::SupplementaryVolumeDescriptor,
      3 => VolumeDescriptorType::VolumePartitionDescriptor,
      255 => VolumeDescriptorType::VolumeDescriptorSetTerminator,
      other => VolumeDescriptorType::Other(other),
    }
  }
}

#[derive(Debug, Clone)]
pub enum VolumeDescriptorIdentifier {
  /// ISO 9660 file system.
  Cd001,
  /// Extended descriptor section.
  Bea01,
  /// UDF file system.
  Nsr02,
  /// UDF file system.
  Nsr03,
  /// Boot loader location and entry point address.
  Boot2,
  /// Denotes the end of the extended descriptor section.
  Tea01,
}

impl VolumeDescriptorIdentifier {
  pub(crate) fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> {
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
}

bitflags::bitflags! {
  #[derive(Debug, Clone)]
  pub struct FileFlags: u8 {
    const EXISTENCE = 1 << 0;
    const DIRECTORY = 1 << 1;
    const ASSOCIATED_FILE = 1 << 2;
    const RECORD = 1 << 3;
    const PROTECTION = 1 << 4;
    const MULTI_EXTENT = 1 << 7;
  }

  #[derive(Debug, Clone)]
  pub struct SupplementaryVolumeFlags: u8 {
    /// Indicates that the escape sequences field within `SupplementaryVolumeDescriptor`
    /// specifies escape sequences registered according to ISO/IEC 2375.
    ///
    /// If this bit is toggled to true, there will be at least one escape sequence
    /// not registered according to ISO/IEC 2375.
    const ESCAPE_SEQUENCES_COMPLIANT = 1 << 0;
    const RESERVED_2 = 1 << 1;
    const RESERVED_3 = 1 << 2;
    const RESERVED_4 = 1 << 3;
    const RESERVED_5 = 1 << 4;
    const RESERVED_6 = 1 << 5;
    const RESERVED_7 = 1 << 6;
  }

  #[derive(Debug, Clone)]
  pub struct BootMediaType: u8 {
    /// 1.2 MiB floppy 
    const FLOPPY_120M = 1 << 0;
    /// 1.44 MiB floppy
    const FLOPPY_144M = 1 << 1;
    /// 2.88 MiB floppy 
    const FLOPPY_288M = 1 << 2;
    const HARD_DISK = 1 << 3;
    // const RESERVED = 1 << 4;
    /// Continued in the next section of the boot catalog.
    const CONTINUATION_ENTRY_FOLLOWS = 1 << 5;
    /// Image contains an ATAPI driver
    const ATAPI_DRIVER = 1 << 6;
    /// Image contains SCSI drivers 
    const SCSI_DRIVERS = 1 << 7;
  }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PlatformId {
  X86 = 0,
  PowerPc = 1,
  Mac = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum BootIndicator {
  NotBootable = 88,
  Bootable = 00,
}

#[derive(Debug, Clone)]
pub struct PrimaryVolumeDescriptor {
  /// Standard identifier.
  pub standard_identifier: VolumeDescriptorIdentifier,
  /// Version. (Always `0x01`).
  pub version: u8,
  /// Name of the system that can act upon sectors `0x00` to `0x0F` for the volume.
  pub system_identifier: String,
  /// Identification of this volume.
  pub volume_identifier: String,
  /// Number of Logical Blocks in which the volume is recorded.
  pub volume_space_size: u32,
  /// The size of the set in this logical volume (number of disks).
  pub volume_set_size: u16,
  /// The number of this disk in the Volume Set.
  pub volume_sequence_number: u16,
  /// The size in bytes of a logical block. NB: This means that
  /// a logical block on a CD could be something other than 2 KiB!
  pub logical_block_size: u16,
  /// The size in bytes of the path table.
  pub path_table_size: u32,
  /// Location of Type-L Path Table. The path table to contains only little-endian values.
  pub type_l_path_table_lba: u32,
  /// LBA location of the optional path table. The path table
  /// pointed to contains only little-endian values. Zero means
  /// that no optional path table exists.
  pub optional_type_l_path_table_lba: u32,
  /// LBA location of the path table. The path tbale pointed to contains
  /// only big-endian values.
  pub type_m_path_table_lba: u32,
  /// LBA location of the optional path table. The path table pointed
  /// to contains only big-endian values. Zero means that no optional path table exists.
  pub optional_type_m_path_table_lba: u32,
  pub root_directory_record: DirectoryRecord,
  /// Identifier of the volume set of which this volume is a member.
  pub volume_set_identifier: String,
  /// The volume publisher. For extended publisher information, the first byte should
  /// be `0x5F`, followed by the filename of a file in the root directory. If not
  /// specified, all bytes should be `0x20`.
  pub publisher_identifier: String,
  /// The identifier of the person(s) who prepared the data for this volume. For
  /// extended preparation information, the first byte should be `0x5F`, followed
  /// by the filename of a file in the root directory. If not specified, all bytes
  /// should be `0x20`.
  pub data_preparer_identifier: String,
  /// Identifies how the data is recorded on this volume. For extended information, the
  /// first byte should be `0x5F`, followed by the filename of a file in the root directory.
  /// If not specified all bytes should be `0x20`.
  pub application_identifier: String,
  /// Filename of a file in the root diretory that contains copyright information for this volume
  /// set. If not specified, all bytes should be 0x20.
  pub copyright_file_identifier: String,
  /// Filename of a file in the root directory that contains abstract information for this volum
  /// set. If not specified, all bytes should be `0x20`.
  pub abstract_file_identifier: [u8; 36],
  /// Filename of a file in the root directory that contains bibliographic information
  /// for this volume set. If not specified all bytes should be `0x20`.
  pub bibliographic_file_identifier: [u8; 37],
  /// The date and time of when the volume was created
  pub volume_creation_date: IsoPreciseDateTime,
  /// The date and time of when the volume was last modified.
  pub volume_modification_date: IsoPreciseDateTime,
  /// The date and time after which this volume is considered to be obsolete. If not specified
  /// then the volume is never considered to be obsolete.
  pub volume_expiration_date: IsoPreciseDateTime,
  /// The date and time after which the volume may be used. IF not specified, the volume may
  /// be used immediately.
  pub volume_effective_date: IsoPreciseDateTime,
  /// The directory records and path table version (always `0x01`).
  pub file_structure_version: u8,
  pub application_data: [u8; 512],
  pub reserved: [u8; 653],
}

#[derive(Debug, Clone)]
pub struct SupplementaryVolumeDescriptor {
  pub standard_identifier: VolumeDescriptorIdentifier,
  pub version: u8,
  pub volume_flags: SupplementaryVolumeFlags,
  pub system_identifier: String,
  pub volume_identifier: String,
  pub volume_space_size: u32,
  pub escape_sequences: [u8; 32],
  pub volume_set_size: u16,
  pub volume_sequence_number: u16,
  pub logical_block_size: u16,
  pub path_table_size: u32,
  pub type_l_path_table_lba: u32,
  pub optional_type_l_path_table_lba: u32,
  pub type_m_path_table_lba: u32,
  pub optional_type_m_path_table_lba: u32,
  pub root_directory_record: DirectoryRecord,
  pub volume_set_identifier: String,
  pub publisher_identifier: String,
  pub data_preparer_identifier: String,
  pub application_identifier: String,
  pub copyright_file_identifier: String,
  pub abstract_file_identifier: [u8; 36],
  pub bibliographic_file_identifier: [u8; 37],
  pub volume_creation_date: IsoPreciseDateTime,
  pub volume_modification_date: IsoPreciseDateTime,
  pub volume_expiration_date: IsoPreciseDateTime,
  pub volume_effective_date: IsoPreciseDateTime,
  pub file_structure_version: u8,
  pub application_data: [u8; 512],
  pub reserved: [u8; 653],
}

#[derive(Debug, Clone)]
pub struct DirectoryRecord {
  pub record_length: u8,
  pub extended_attribute_record_length: u8,
  pub extent_lba: u32,
  pub extent_length: u32,
  pub recording_date: IsoDateTime,
  pub file_flags: FileFlags,
  pub file_unit_size: u8,
  pub interleave_gap_size: u8,
  pub volume_sequence_number: u16,
  pub identifier_length: u8,
  pub identifier: String,
}

impl DirectoryRecord {
  pub fn is_directory(&self) -> bool {
    self.file_flags.contains(FileFlags::DIRECTORY)
  }
}

#[derive(Debug)]
pub enum VolumeDescriptor {
  Primary(PrimaryVolumeDescriptor),
  Supplementary(SupplementaryVolumeDescriptor),
  Boot(BootRecord),
}

#[derive(Debug)]
pub struct PathTableRecord {
  pub directory_identifier_length: u8,
  pub extended_attribute_record_length: u8,
  pub extent_lba: u32,
  pub parent_directory_number: u16,
  pub directory_identifier: String,
}

#[derive(Debug)]
pub struct BootRecord {
  pub standard_identifier: VolumeDescriptorIdentifier,
  pub version: u8,
  pub boot_system_identifier: String,
  pub boot_identifier: String,
  pub boot_system_use: [u8; 1977],
}

#[derive(Debug, Clone)]
pub struct BootCatalog {
  pub header_id: u8,
  pub platform_id: PlatformId,
  pub identifier: String,
  pub checksum: u16,
  pub bootable: BootIndicator,
  pub boot_media_type: BootMediaType,
  pub load_segment: u16,
  pub system_type: u8,
  pub sector_count: u16,
  pub load_rba: u32,
}
