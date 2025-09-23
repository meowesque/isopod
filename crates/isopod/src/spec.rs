//! ISO 9660 specification types including extensions such as Joliet and Rock Ridge.

pub trait Extension {
  type FileFlags: std::fmt::Debug;
  type FileIdentifier: std::fmt::Debug;
  type DirectoryIdentifier: std::fmt::Debug;
}

/// No extensions; Standard ISO 9660 only.
#[derive(Debug)]
pub struct NoExtension;

impl Extension for NoExtension {
  type FileFlags = FileFlags;
  type FileIdentifier = FileIdentifier<32>;
  type DirectoryIdentifier = DirectoryIdentifier<31>;
}

#[derive(Debug)]
pub enum JolietLevel {
  Level1,
  Level2,
  Level3,
}

/// Microsoft Joliet extension.
#[derive(Debug)]
pub struct JolietExtension {
  pub level: JolietLevel,
}

impl Extension for JolietExtension {
  type FileFlags = (); // TODO(meowesque): Define Joliet-specific file flags.
  type FileIdentifier = JolietFileIdentifier;
  type DirectoryIdentifier = JolietDirectoryIdentifier;
}

/// `[\s\!\"\%\&\'\(\)\*\+\,\-\.\/0-9A-Z\:\;\<\=\>\?\_A-Z0-9]`
#[derive(Debug)]
pub struct ACharacters<const LENGTH: usize>([u8; LENGTH]);

/// `[0-9A-Z_]``
#[derive(Debug)]
pub struct DCharacters<const LENGTH: usize>([u8; LENGTH]);

#[derive(Debug)]
pub struct A1Characters<const LENGTH: usize>([u8; LENGTH]);

#[derive(Debug)]
pub struct D1Characters<const LENGTH: usize>([u8; LENGTH]);

/// Escape sequences conforming to ISO/IEC 2022, including the escape characters.
///
/// If all the bytes of the escape sequences are zero, it shall mean that the set
/// of a1-characters is identical to the set of a-characters.
#[derive(Debug)]
pub struct EscapeSequences<const LENGTH: usize>([u8; LENGTH]);

/// Escape sequences conforming to ISO/IEC 2022, excluding the escape characters.
#[derive(Debug)]
pub struct VariadicEscapeSequences(Vec<u8>);

#[derive(Debug)]
pub struct JolietFileIdentifier([u16; 64]);

#[derive(Debug)]
pub struct JolietDirectoryIdentifier([u16; 64]);

bitflags::bitflags! {
  #[derive(Debug)]
  pub struct FileFlags: u8 {
    const EXISTENCE = 1 << 0;
    const DIRECTORY = 1 << 1;
    const ASSOCIATED_FILE = 1 << 2;
    const RECORD = 1 << 3;
    const PROTECTION = 1 << 4;
    const RESERVED_5 = 1 << 5;
    const RESERVED_6 = 1 << 6;
    const MULTI_EXTENT = 1 << 7;
  }

  #[derive(Debug)]
  pub struct Permissions: u16 {
    const SYSTEM_READ = 1 << 0;
    /// "Shall be set to 1."
    const PERMISSION_1 = 1 << 1;
    const SYSTEM_EXECUTE = 1 << 2;
    /// "Shall be set to 1."
    const PERMISSION_3 = 1 << 3;
    const USER_READ = 1 << 4;
    /// "Shall be set to 1."
    const PERMISSION_5 = 1 << 5;
    const USER_EXECUTE = 1 << 6;
    /// "Shall be set to 1."
    const PERMISSION_7 = 1 << 7;
    const OTHER_READ = 1 << 8;
    /// "Shall be set to 1."
    const PERMISSION_9 = 1 << 9;
    const OTHER_EXECUTE = 1 << 10;
    /// "Shall be set to 1."
    const PERMISSION_11 = 1 << 11;
    const ALL_READ = 1 << 12;
    /// "Shall be set to 1."
    const PERMISSION_13 = 1 << 13;
    const ALL_EXECUTE = 1 << 14;
    /// "Shall be set to 1."
    const PERMISSION_15 = 1 << 15;
  }

  #[derive(Debug)]
  pub struct VolumeFlags: u8 {
    /// If zero, shall mean that the escape sequences field specifies only
    /// escape sequences registered by ISO/IEC 2375.
    ///
    /// If one, shall mean that the escape sequences field includes
    /// atleast one escape sequence not registered according to ISO/IEC 2375.
    const UNREGISTERED_ESCAPE_SEQUENCES = 1 << 0;
    const RESERVED_1 = 1 << 1;
    const RESERVED_2 = 1 << 2;
    const RESERVED_3 = 1 << 3;
    const RESERVED_4 = 1 << 4;
    const RESERVED_5 = 1 << 5;
    const RESERVED_6 = 1 << 6;
    const RESERVED_7 = 1 << 7;
  }
}

#[derive(Debug)]
pub struct FileIdentifier<const LENGTH: usize>([u8; LENGTH]);

/// `DCharacters`/`D1Characters`.
#[derive(Debug)]
pub struct DirectoryIdentifier<const LENGTH: usize>([u8; LENGTH]);

/// TODO(meowesque): Define this better?
#[derive(Debug)]
pub struct OwnerIdentification(u16);

/// TODO(meowesque): Define this better?
#[derive(Debug)]
pub struct GroupIdentification(u16);

#[repr(u8)]
#[derive(Debug)]
pub enum RecordFormat {
  StructureNotSpecified = 0,
  FixedLengthRecords = 1,
  VariableLengthRecordsMsb = 2,
  VariableLengthRecordsLsb = 3,
  Other(u8),
}

#[repr(u8)]
#[derive(Debug)]
pub enum RecordAttributes {
  PreceededByLfcFollowedByCrc = 0,
  /// First byte of the record shall be interpreted as specified in ISO/IEC 1539-1 for vertical spacing.
  FirstByteInterpretedByIso15391 = 1,
  ContainsNecessaryControlInformation = 2,
  Other(u8),
}

#[repr(u8)]
#[derive(Debug)]
pub enum ExtendedAttributeRecordVersion {
  Standard = 1,
  Other(u8),
}

#[derive(Debug)]
pub enum StandardIdentifier {
  /// Standard ISO 9660 identifier; "CD001"
  Cd001,
  /// Denotes the beginning of the extended descriptor section; "BEA01"
  Bea01,
  /// Indicates that this volume contains a UDF filesystem; "NSR02"
  Nsr02,
  /// Indicates that this volume contains a UDF filesystem; "NSR03"
  Nsr03,
  /// Includes information concerning boot loader location and entry point address; "BOOT2"
  Boot2,
  /// Indicates the end of the extended descriptor section; "TEA01"
  Tea01,
  /// Any other identifier not covered by the above variants.
  Other([u8; 5]),
}

#[derive(Debug)]
#[repr(u8)]
pub enum VolumeDescriptorVersion {
  Standard = 1,
  Other(u8),
}

#[derive(Debug)]
#[repr(u8)]
pub enum FileStructureVersion {
  Standard = 1,
  Other(u8),
}

#[derive(Debug)]
pub struct DigitsYear(u16);

#[derive(Debug)]
pub struct DigitsMonth(u8);

#[derive(Debug)]
pub struct DigitsDay(u8);

#[derive(Debug)]
pub struct DigitsHour(u8);

#[derive(Debug)]
pub struct DigitsMinute(u8);

#[derive(Debug)]
pub struct DigitsSecond(u8);

#[derive(Debug)]
pub struct NumericalYear(u8);

#[derive(Debug)]
pub struct NumericalMonth(u8);

#[derive(Debug)]
pub struct NumericalDay(u8);

#[derive(Debug)]
pub struct NumericalHour(u8);

#[derive(Debug)]
pub struct NumericalMinute(u8);

#[derive(Debug)]
pub struct NumericalSecond(u8);

#[derive(Debug)]
pub struct NumericalGmtOffset(i8);

#[derive(Debug)]
pub struct DigitsDate {
  pub year: DigitsYear,
  pub month: DigitsMonth,
  pub day: DigitsDay,
  pub hour: DigitsHour,
  pub minute: DigitsMinute,
  pub second: DigitsSecond,
  pub gmt_offset: NumericalGmtOffset,
}

#[derive(Debug)]
pub struct NumericalDate {
  pub year: NumericalYear,
  pub month: NumericalMonth,
  pub day: NumericalDay,
  pub hour: NumericalHour,
  pub minute: NumericalMinute,
  pub second: NumericalSecond,
  pub gmt_offset: NumericalGmtOffset,
}

#[derive(Debug)]
pub struct PrimaryVolumeDescriptor<Ext: Extension> {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub system_identifier: ACharacters<32>,
  pub volume_identifier: DCharacters<32>,
  pub volume_space_size: u32,
  pub volume_set_size: u16,
  pub volume_sequence_number: u16,
  pub logical_block_size: u16,
  pub path_table_size: u32,
  pub type_l_path_table_location: u32,
  pub optional_type_l_path_table_location: u32,
  pub type_m_path_table_location: u32,
  pub optional_type_m_path_table_location: u32,
  pub root_directory_record: RootDirectoryRecord<Ext>,
  pub volume_set_identifier: DCharacters<128>,
  pub publisher_identifier: ACharacters<128>,
  pub data_preparer_identifier: ACharacters<128>,
  pub application_identifier: ACharacters<128>,
  pub copyright_file_identifier: DCharacters<38>, // Separator 1 and 2
  pub abstract_file_identifier: DCharacters<36>,  // Separator 1 and 2
  pub bibliographic_file_identifier: DCharacters<37>, // Separator 1 and 2
  pub creation_date: DigitsDate,
  pub modification_date: DigitsDate,
  pub expiration_date: DigitsDate,
  pub effective_date: DigitsDate,
  pub file_structure_version: FileStructureVersion,
  pub application_use: [u8; 512],
}

#[derive(Debug)]
pub struct SupplementaryVolumeDescriptor<Ext: Extension> {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub volume_flags: VolumeFlags,
  pub system_identifier: A1Characters<32>,
  pub volume_identifier: D1Characters<32>,
  pub volume_space_size: u32,
  pub escape_sequences: EscapeSequences<32>,
  pub volume_set_size: u16,
  pub volume_sequence_number: u16,
  pub logical_block_size: u16,
  pub path_table_size: u32,
  pub type_l_path_table_location: u32,
  pub optional_type_l_path_table_location: u32,
  pub type_m_path_table_location: u32,
  pub optional_type_m_path_table_location: u32,
  pub root_directory_record: RootDirectoryRecord<Ext>,
  pub volume_set_identifier: D1Characters<128>,
  pub publisher_identifier: A1Characters<128>,
  pub data_preparer_identifier: A1Characters<128>,
  pub application_identifier: A1Characters<128>,
  pub copyright_file_identifier: D1Characters<38>, // Separator 1 and 2
  pub abstract_file_identifier: D1Characters<36>,  // Separator 1 and
  pub bibliographic_file_identifier: D1Characters<37>, // Separator 1 and 2
  pub creation_date: DigitsDate,
  pub modification_date: DigitsDate,
  pub expiration_date: DigitsDate,
  pub effective_date: DigitsDate,
  pub file_structure_version: FileStructureVersion,
  pub application_use: [u8; 512],
}

#[derive(Debug)]
pub struct VolumePartitionDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub system_identifier: ACharacters<32>,
  pub volume_partition_identifier: DCharacters<32>,
  pub volume_partition_location: u32,
  pub volume_partition_size: u32,
}

#[derive(Debug)]
pub struct DirectoryRecord<Ext: Extension> {
  pub length: u8,
  pub extended_attribute_length: u8,
  pub extent_location: u32,
  pub data_length: u32,
  pub recording_date: NumericalDate,
  pub file_flags: Ext::FileFlags,
  pub file_unit_size: u8,
  pub interleave_gap_size: u8,
  pub volume_sequence_number: u16,
  pub file_identifier_length: u8,
  pub file_identifier: Ext::FileIdentifier,
}

/// Root directory record as found in `SupplementaryVolumeDescriptor` and
/// `PrimaryVolumeDescriptor`. Like `DirectoryRecord` but without the `length`
/// and `extended_attribute_length` fields.
#[derive(Debug)]
pub struct RootDirectoryRecord<Ext: Extension> {
  pub extent_location: u32,
  pub data_length: u32,
  pub recording_date: NumericalDate,
  pub file_flags: Ext::FileFlags,
  pub file_unit_size: u8,
  pub interleave_gap_size: u8,
  pub volume_sequence_number: u16,
}

#[derive(Debug)]
pub struct PathTableRecord<Ext: Extension> {
  pub directory_identifier_length: u8,
  pub extent_location: u32,
  pub parent_directory_number: u16,
  pub directory_identifier: Ext::DirectoryIdentifier,
}

#[derive(Debug)]
pub struct ExtendedAttributeRecord {
  pub owner_identification: OwnerIdentification,
  pub group_identification: GroupIdentification,
  pub permissions: Permissions,
  pub file_creation_date: DigitsDate,
  pub file_modification_date: DigitsDate,
  pub file_expiration_date: DigitsDate,
  pub file_effective_date: DigitsDate,
  pub record_format: RecordFormat,
  pub record_attributes: RecordAttributes,
  pub extended_attribute_record_version: ExtendedAttributeRecordVersion,
  pub application_use: Vec<u8>,
  pub escape_sequences: VariadicEscapeSequences,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ElToritoHeaderId {
  Standard = 1,
  Other(u8),
}

#[derive(Debug)]
#[repr(u8)]
pub enum ElToritoPlatformId {
  X86 = 0,
  PowerPc = 1,
  Mac = 2,
  Other(u8),
}

#[derive(Debug)]
#[repr(u8)]
pub enum ElToritoBootIndicator {
  Bootable = 0x88,
  NonBootable = 0x00,
  Other(u8),
}

#[derive(Debug)]
pub struct ElToritoManufacturerId([u8; 16]);

bitflags::bitflags! {
  #[derive(Debug)]
  pub struct ElToritoBootMediaType: u8 {
    // TODO(meowesque): Bitfields
  }

  #[derive(Debug)]
  pub struct ElToritoBootMediaTypeExt: u8 {
    // TODO(meowesque): Bitfields
    const CONTINUATION_ENTRY_FOLLOWS = 1 << 5;
    const CONTAINS_ATAPI_DRIVER = 1 << 6;
    const CONTAINS_SCSI_DRIVERS = 1 << 7;
  }

  #[derive(Debug)]
  pub struct ElToritoExtensionRecordFollowsIndicator: u8 {
    const EXTENSION_RECORD_FOLLOWS = 1 << 5;
  }
}

#[repr(u8)]
#[derive(Debug)]
pub enum ElToritoHeaderIndicator {
  MoreHeadersFollow = 90,
  FinalHeader = 91,
}

#[derive(Debug)]
pub struct ElToritoSectionId([u8; 16]);

#[derive(Debug)]
#[repr(u8)]
pub enum ElToritoSelectionCriteriaType {
  NoSelectionCriteria = 0,
  LanguageAndVersionInformation = 1,
  Other(u8),
}

#[derive(Debug)]
pub struct ElToritoInitialSectionEntry {
  pub boot_indicator: ElToritoBootIndicator,
  pub boot_media_type: ElToritoBootMediaType,
  pub load_segment: u16,
  pub system_type: u8,
  pub sector_count: u16,
  pub virtual_disk_location: u32,
}

#[derive(Debug)]
pub struct ElToritoSectionHeaderEntry {
  pub header_indicator: ElToritoHeaderIndicator,
  pub platform_id: ElToritoPlatformId,
  pub succeeding_section_entries: u16,
  pub section_id: ElToritoSectionId,
}

#[derive(Debug)]
pub struct ElToritoValidationEntry {
  pub header_id: ElToritoHeaderId,
  pub platform_id: ElToritoPlatformId,
  pub manufacturer_id: ElToritoManufacturerId,
  pub checksum: u16,
}

#[derive(Debug)]
pub struct ElToritoSectionEntry {
  pub boot_indicator: ElToritoBootIndicator,
  pub boot_media_type: ElToritoBootMediaTypeExt,
  pub load_segment: u16,
  pub system_type: u8,
  pub sector_count: u16,
  pub virtual_disk_location: u32,
  pub selection_criteria_type: ElToritoSelectionCriteriaType,
  pub vendor_selection_criteria: [u8; 31], // TODO(meowesque): Check if this is right
}

#[derive(Debug)]
pub struct ElToritoSelectionEntryExtension {
  pub extension_record_follows_indicator: ElToritoExtensionRecordFollowsIndicator,
  pub vendor_unique_selection_criteria: [u8; 29], // TODO(meowesque): Check if this is right
}

#[derive(Debug)]
pub struct ElToritoBootRecordVolumeDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub boot_catalog_pointer: u32,
}
