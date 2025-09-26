use crate::{
  serialize::IsoSerialize,
  spec,
  writer::{fs::DirectoryLike, volume::VolumeLike},
};

pub mod error;
pub mod fs;
pub mod lba;
pub mod sector;
pub mod volume;

/*
use std::{
  collections::HashSet,
  fs::File,
  io::Seek,
  path::{Path, PathBuf},
};

use crate::{
  serialize::{self, IsoSerialize},
  spec,
};

#[derive(Debug)]
pub enum FsEntry {
  File(FsFile),
  Directory(FsDirectory),
}

impl FsEntry {
  fn into_file(self) -> Option<FsFile> {
    match self {
      Self::File(file) => Some(file),
      _ => None,
    }
  }

  fn name(&self) -> &str {
    match self {
      Self::File(file) => &file.name,
      Self::Directory(dir) => &dir.name,
    }
  }

  fn lba(&self) -> Option<u32> {
    match self {
      Self::File(file) => file.lba,
      Self::Directory(dir) => dir.lba,
    }
  }

  fn calculate_extent(&self) -> u64 {
    match self {
      Self::File(file) => file.calculate_extent(),
      Self::Directory(dir) => dir.calculate_extent(),
    }
  }

  fn file_flags(&self) -> spec::FileFlags {
    match self {
      Self::File(_) => spec::FileFlags::empty(),
      Self::Directory(_) => spec::FileFlags::DIRECTORY,
    }
  }

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    spec::DirectoryRecord {
      // TODO(meowesque): Make this field optional so we calculate when serialized?
      length: 33 + self.name().len() as u8 + (self.name().len() as u8 % 2),
      extended_attribute_length: 0,
      extent_location: self.lba().expect("LBA must be assigned"),
      data_length: self.calculate_extent() as u32,
      recording_date: chrono::Utc::now().into(),
      file_flags: self.file_flags(),
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 0,
      file_identifier_length: self.name().len() as u8,
      file_identifier: spec::FileIdentifier::from_bytes_truncated(self.name().as_bytes()),
    }
  }
}

#[derive(Debug)]
pub enum FsFileContent {
  Handle {
    metadata: std::fs::Metadata,
    file: std::fs::File
  },
}

impl FsFileContent {
  fn size(&self) -> u64 {
    match self {
      Self::Handle { metadata, .. } => metadata.len(),
    }
  }
}

#[derive(Debug)]
pub struct FsFile {
  lba: Option<u32>,
  name: String,
  /// Cached size. Since to check a file handle's metadata requires side effects.
  size: u64,
  content: FsFileContent,
}

impl FsFile {
  fn new(name: String, content: FsFileContent) -> Result<Self, Error> {
    Ok(Self {
      lba: None,
      name,
      size: content.size()?,
      content,
    })
  }

  fn calculate_extent(&self) -> u64 {
    self.size
  }
}

#[derive(Debug)]
pub struct FsDirectory {
  lba: Option<u32>,
  name: String,
  entries: Vec<FsEntry>,
}

impl FsDirectory {
  fn calculate_extent(&self) -> u64 {
    33 + self.name.len() as u64 + (self.name.len() as u64 % 2)
  }

  fn find_mut(&mut self, name: impl AsRef<str>) -> Option<&mut FsEntry> {
    self
      .entries
      .iter_mut()
      .find(|entry| entry.name() == name.as_ref())
  }

  fn contains_file(&self, name: impl AsRef<str>) -> bool {
    self.entries.iter().any(|entry| match entry {
      FsEntry::File(file) => file.name == name.as_ref(),
      _ => false,
    })
  }

  fn contains_directory(&self, name: impl AsRef<str>) -> bool {
    self.entries.iter().any(|entry| match entry {
      FsEntry::Directory(dir) => dir.name == name.as_ref(),
      _ => false,
    })
  }

  fn upsert(&mut self, entry: FsEntry) {
    match (self.find_mut(entry.name()), entry) {
      // If a file with the same name exists, replace it.
      (Some(FsEntry::File(dup)), FsEntry::File(file)) => {
        let _ = std::mem::replace(dup, file);
      }
      // If a directory with the same name exists, upsert entries.
      (Some(FsEntry::Directory(dup)), FsEntry::Directory(dir)) => {
        dir.entries.into_iter().for_each(|x| dup.upsert(x))
      }
      // Otherwise, just add the entry.
      (_, entry) => self.entries.push(entry),
    }
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
  lba: Option<u32>,
  entries: Vec<FsEntry>,
}

impl Filesystem {
  pub fn upsert(&mut self, entry: FsEntry) {
    // TODO(meowesque): Deduplicate code.
    match (
      self.entries.iter_mut().find(|e| e.name() == entry.name()),
      entry,
    ) {
      // If a file with the same name exists, replace it.
      (Some(FsEntry::File(dup)), FsEntry::File(file)) => {
        let _ = std::mem::replace(dup, file);
      }
      // If a directory with the same name exists, upsert entries.
      (Some(FsEntry::Directory(dup)), FsEntry::Directory(dir)) => {
        dir.entries.into_iter().for_each(|x| dup.upsert(x))
      }
      // Otherwise, just add the entry.
      (_, entry) => self.entries.push(entry),
    }
  }

  pub fn upsert_file(
    &mut self,
    path: impl AsRef<Path>,
    content: FsFileContent,
  ) -> Result<(), Error> {
    let path = path.as_ref();
    let components = path.components();

    let mut tail = FsEntry::File(FsFile::new(
      // TODO(meowesque): Handle error more gracefully.
      path
        .file_name()
        .expect("Must have a filename")
        .to_string_lossy()
        .to_string(),
      content,
    )?);

    for component in components.rev().skip(1) {
      tail = FsEntry::Directory(FsDirectory {
        lba: None,
        name: component.as_os_str().to_string_lossy().to_string(),
        entries: vec![tail],
      });
    }

    self.upsert(tail);

    Ok(())
  }

  fn calculate_extent(&self) -> u64 {
    self
      .entries
      .iter()
      .map(|e| e.calculate_extent())
      .sum::<u64>()
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

  fn write<W>(&self, mut writer: W) -> Result<(), Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    // TODO(meowesque): Propagate a writer wrapper or something.
    let _ = writer.seek(std::io::SeekFrom::Start(self.lba.unwrap() as u64 * 2048))?;

    // TODO(meowesque): This wont work for directories with many entries.
    let mut bytes_offset = 0;

    for entry in &self.entries {
      match entry {
        FsEntry::File(file) => {
          file.write(&mut writer)?;
        }
        FsEntry::Directory(dir) => {
          dir.write(&mut writer)?;
        }
      }

      let descriptor = entry.descriptor();
      let mut bytes = [0u8; 2048];
      descriptor.serialize(&mut bytes)?;

      writer.seek(std::io::SeekFrom::Start(
        self.lba.unwrap() as u64 * 2048 + bytes_offset as u64,
      ))?;
      writer.write_all(&bytes[..descriptor.extent() as usize])?;
      bytes_offset += descriptor.extent() as usize;
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct PrimaryVolume {
  pub volume_id: String,
  pub publisher: Option<String>,
  pub preparer: Option<String>,
  pub filesystem: Filesystem,
}

impl PrimaryVolume {
  fn create_descriptor(&self) -> spec::PrimaryVolumeDescriptor {
    todo!()
  }
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

  pub fn write<W>(&mut self, mut writer: W) -> Result<(), Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    let lba_offset = self.volumes.len() as u32 + 16;
    let mut allocator = LbaAllocator::new(self.options.sector_size as u32, lba_offset);

    let standard_identifier = self.options.standard.standard_identifier();

    for volume in self.volumes.iter_mut() {
      match volume {
        Volume::Primary(pv) => {
          pv.filesystem.assign_lbas(&mut allocator);

          let d = spec::PrimaryVolumeDescriptor {
            standard_identifier: standard_identifier.clone(),
            version: spec::VolumeDescriptorVersion::Standard,
            system_identifier: spec::ACharacters::from_bytes_truncated(b"LINUX"),
            volume_identifier: spec::DCharacters::from_bytes_truncated(pv.volume_id.as_bytes()),
            volume_space_size: 0,
            volume_set_size: 0,
            volume_sequence_number: 0,
            logical_block_size: self.options.sector_size as u16,
            path_table_size: 0,
            type_l_path_table_location: pv.filesystem.lba.expect("LBA must be assigned"),
            optional_type_l_path_table_location: 0,
            type_m_path_table_location: 0,
            optional_type_m_path_table_location: 0,
            root_directory_record: spec::RootDirectoryRecord {
              extent_location: 0,
              data_length: 0,
              recording_date: chrono::Utc::now().into(),
              file_flags: spec::FileFlags::DIRECTORY,
              file_unit_size: 0,
              interleave_gap_size: 0,
              volume_sequence_number: 0,
            },
            volume_set_identifier: spec::DCharacters::from_bytes_truncated(b"abc"),
            publisher_identifier: spec::ACharacters::from_bytes_truncated(b"hi noxie (:"),
            data_preparer_identifier: spec::ACharacters::from_bytes_truncated(b"def"),
            application_identifier: spec::ACharacters::from_bytes_truncated(b"ghi"),
            copyright_file_identifier: spec::DCharacters::from_bytes_truncated(b"jkl"),
            abstract_file_identifier: spec::DCharacters::from_bytes_truncated(b"mno"),
            bibliographic_file_identifier: spec::DCharacters::from_bytes_truncated(b"pqr"),
            creation_date: chrono::Utc::now().into(),
            modification_date: chrono::Utc::now().into(),
            expiration_date: chrono::Utc::now().into(),
            effective_date: chrono::Utc::now().into(),
            file_structure_version: spec::FileStructureVersion::Standard,
            application_use: [0; 512],
          };

          let mut bytes = [0u8; 2048];

          d.serialize(&mut bytes)?;

          writer.seek(std::io::SeekFrom::Start(0)).unwrap();
          writer.write_all(&[0u8; 16 * 2048])?;
          writer.write_all(&bytes)?;

          spec::VolumeDescriptorSetTerminator
            .serialize(&mut bytes)
            .unwrap();
          writer.write_all(&bytes)?;

          pv.filesystem.write(&mut writer)?;

          dbg!(pv);
        }
      }
    }

    Ok(())
  }
}
*/

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

pub struct WriterOptions {
  pub sector_size: u16,
  pub standard: Standard,
}

pub struct IsoWriter {
  options: WriterOptions,
  volumes: Vec<volume::Volume>,
}

impl IsoWriter {
  pub fn new(options: WriterOptions) -> Self {
    Self {
      options,
      volumes: vec![],
    }
  }

  pub fn add_volume(&mut self, volume: impl Into<volume::Volume>) {
    self.volumes.push(volume.into());
  }

  pub fn write<W>(&mut self, mut writer: W) -> Result<(), error::Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    fn write_file_entry<W>(
      mut writer: W,
      file_entry: &fs::FileEntry,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
    {
      let mut reader = std::io::BufReader::new(&file_entry.handle);

      writer.seek(std::io::SeekFrom::Start(
        file_entry.extent_lba.unwrap() as u64 * sector_size,
      ))?;
      std::io::copy(&mut reader, &mut writer)?;

      Ok(())
    }

    fn write_directory_entry<W, D>(
      mut writer: W,
      directory_entry: &D,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
      D: fs::DirectoryLike + fs::EntryLike,
    {
      let mut sector_writer = sector::SectorWriter::new(
        &mut writer,
        directory_entry.extent_lba().unwrap() as u64,
        sector_size,
      );

      // TODO(meowesque): Write . and .. entries.

      let mut byte_buf = vec![];

      for entry in directory_entry.entries_iter() {
        let entry_descriptor = entry.descriptor();

        byte_buf.resize(entry_descriptor.extent(), 0);
        entry_descriptor.serialize(&mut byte_buf[..])?;

        sector_writer.write_aligned(&byte_buf[..entry_descriptor.extent() as usize])?;
      }

      for entry in directory_entry.entries_iter() {
        write_entry(&mut writer, entry, sector_size)?;
      }

      Ok(())
    }

    fn write_entry<W>(
      writer: &mut W,
      entry: &fs::Entry,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
    {
      match entry {
        fs::Entry::File(file_entry) => write_file_entry(writer, file_entry, sector_size),
        fs::Entry::Directory(dir_entry) => write_directory_entry(writer, dir_entry, sector_size),
      }
    }

    let mut allocator = lba::LbaAllocator::new(
      self.options.sector_size as u32,
      self.volumes.len() as u32 + 16,
    );

    let context = volume::VolumeContext {
      sector_size: self.options.sector_size as u32,
      standard_identifier: self.options.standard.standard_identifier(),
    };

    {
      let mut bytes: [u8; 2048] = [0; 2048];

      writer.seek(std::io::SeekFrom::Start(0))?;

      for volume in self.volumes.iter_mut() {
        match volume {
          volume::Volume::Primary(pv) => {
            pv.filesystem.assign_extent_lbas(&mut allocator);
            pv.descriptor(&context).serialize(&mut bytes)?;
            writer.write_all(&bytes)?;
            write_directory_entry(&mut writer, &pv.filesystem.root, context.sector_size as u64)?;
          }
        }
      }
    }

    Ok(())
  }
}
