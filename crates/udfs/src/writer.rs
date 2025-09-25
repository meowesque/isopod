use std::{collections::HashSet, path::PathBuf};

use crate::serialize;

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
  pub fn new(sector_size: u32, start_lba: u32) -> Self {
    Self {
      sector_size,
      next_lba: start_lba,
    }
  }

  pub fn allocate(&mut self, size: u32) -> u32 {
    let lba = self.next_lba;
    let sectors = (size + self.sector_size - 1) / self.sector_size;
    self.next_lba += sectors;
    lba
  }
}

pub enum FsEntry {
  File(FsFile),
  Directory(FsDirectory),
}

pub enum FsFileContent {
  Handle(std::fs::File)
}

pub struct FsFile {
  pub name: String, 
  pub content: FsFileContent,
}

pub struct FsDirectory {
  pub name: String,
  pub entries: Vec<FsEntry>,
}

pub struct Filesystem {
  pub entries: Vec<FsEntry>,
}

pub struct PrimaryVolume {
  pub volume_id: String,
  pub publisher: Option<String>,
  pub preparer: Option<String>,
  pub filesystem: Filesystem,
}

pub enum Volume {
  Primary(PrimaryVolume)
}

#[derive(Debug)]
pub struct WriterOptions {}

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

  pub fn write<W>(&self, writer: W) -> Result<(), Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    todo!()
  }
}
