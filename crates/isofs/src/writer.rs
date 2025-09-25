use std::{collections::HashSet, fs::File, path::PathBuf};

use crate::{serialize, spec};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Serialization error: {0}")]
  Serialize(#[from] serialize::IsoSerializeError),
}

struct LbaAllocator {
  sector_size: u32,
  next_lba: u32,
}

impl LbaAllocator {
  fn new(sector_size: u32, start_lba: u32) -> Self {
    Self {
      sector_size,
      next_lba: start_lba,
    }
  }

  fn allocate(&mut self, size: u32) -> u32 {
    let lba = self.next_lba;
    let sectors = (size + self.sector_size - 1) / self.sector_size;
    self.next_lba += sectors;
    lba
  }
}

#[derive(Debug)]
enum FsEntry {
  File(FsFile),
  Directory(FsDirectory),
}

#[derive(Debug)]
enum FsFileContent {
  Handle(std::fs::File),
}

#[derive(Debug)]
struct FsFile {
  lba: Option<u32>,
  name: String,
  content: FsFileContent,
}

impl FsFile {
  fn calculate_extent(&self) -> u64 {
    match &self.content {
      FsFileContent::Handle(f) => f.metadata().map(|m| m.len()).unwrap_or(0),
    }
  }
}

#[derive(Debug)]
struct FsDirectory {
  lba: Option<u32>,
  name: String,
  entries: Vec<FsEntry>,
}

impl FsDirectory {
  fn calculate_extent(&self) -> u64 {
    33 + self.name.len() as u64 + (self.name.len() as u64 % 2)
  }

  fn add_entry(&mut self, entry: FsEntry) {
    self.entries.push(entry);
  }

  fn assign_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.lba = Some(allocator.allocate(self.calculate_extent() as u32));

    for entry in &mut self.entries {
      match entry {
        FsEntry::File(file) => {
          file.lba = Some(allocator.allocate(file.calculate_extent() as u32));
        }
        FsEntry::Directory(directory) => {
          directory.assign_lbas(allocator);
        }
      }
    }
  }
}

#[derive(Default, Debug)]
pub struct Filesystem {
  pub entries: Vec<FsEntry>,
}

impl Filesystem {
  fn assign_lbas(&mut self, allocator: &mut LbaAllocator) {
    for entry in &mut self.entries {
      match entry {
        FsEntry::File(file) => {
          file.lba = Some(allocator.allocate(file.calculate_extent() as u32));
        }
        FsEntry::Directory(directory) => {
          directory.assign_lbas(allocator);
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct PrimaryVolume {
  pub volume_id: String,
  pub publisher: Option<String>,
  pub preparer: Option<String>,
  pub filesystem: Filesystem,
}

pub enum Volume {
  Primary(PrimaryVolume),
}

#[derive(Debug)]
pub enum Standard {
  Iso9660,
}

impl Standard {
  fn standard_identifier(&self) -> spec::StandardIdentifier {
    match self {
      Standard::Iso9660 => spec::StandardIdentifier::Cd001,
    }
  }
}

#[derive(Debug)]
pub struct WriterOptions {
  pub sector_size: u16,
  pub standard: Standard,
}

pub struct IsoWriter {
  options: WriterOptions,
  volumes: Vec<Volume>,
}

impl IsoWriter {
  pub fn new(options: WriterOptions) -> Self {
    Self {
      options,
      volumes: vec![],
    }
  }

  pub fn add_volume(&mut self, volume: impl Into<Volume>) {
    self.volumes.push(volume.into());
  }

  pub fn write<W>(&mut self, writer: W) -> Result<(), Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    let lba_offset = (self.volumes.len() as u32 + 16) * self.options.sector_size as u32;
    let mut allocator = LbaAllocator::new(self.options.sector_size as u32, lba_offset);

    let standard_identifier = self.options.standard.standard_identifier();

    for volume in self.volumes.iter_mut() {
      match volume {
        Volume::Primary(pv) => {
          pv.filesystem.assign_lbas(&mut allocator);

          let d = spec::PrimaryVolumeDescriptor {
            standard_identifier,
            version: spec::VolumeDescriptorVersion::Standard,
            system_identifier: spec::ACharacters::from_bytes_truncated(b"LINUX"),
            volume_identifier: spec::DCharacters::from_bytes_truncated(pv.volume_id.as_bytes()),
            volume_space_size: 0,
            volume_set_size: 0,
            volume_sequence_number: 0,
            logical_block_size: self.options.sector_size as u16,
            path_table_size: 0,
            type_l_path_table_location: 0,
            optional_type_l_path_table_location: 0,
            type_m_path_table_location: 0,
            optional_type_m_path_table_location: 0,
            root_directory_record: spec::RootDirectoryRecord {
              extent_location: 0,
              data_length: 0,
              recording_date: todo!(),
              file_flags: spec::FileFlags::DIRECTORY,
              file_unit_size: 0,
              interleave_gap_size: 0,
              volume_sequence_number: 0,
            },
            volume_set_identifier: spec::DCharacters::from_bytes_truncated(b""),
            publisher_identifier: spec::ACharacters::from_bytes_truncated(b"erm"),
            data_preparer_identifier: spec::ACharacters::from_bytes_truncated(b""),
            application_identifier: spec::ACharacters::from_bytes_truncated(b""),
            copyright_file_identifier: spec::DCharacters::from_bytes_truncated(b""),
            abstract_file_identifier: spec::DCharacters::from_bytes_truncated(b""),
            bibliographic_file_identifier: spec::DCharacters::from_bytes_truncated(b""),
            creation_date: todo!(),
            modification_date: todo!(),
            expiration_date: todo!(),
            effective_date: todo!(),
            file_structure_version: spec::FileStructureVersion::Standard,
            application_use: [0; 512],
          };

          dbg!(pv);
        }
      }
    }

    todo!()
  }
}
