mod parse;
mod read;
mod spec;
mod write;

use std::{cell::RefCell, rc::Rc};

use parse::Parse;

pub type Result<T, E> = std::result::Result<T, Error<E>>;

#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
  #[error("Parse error")]
  Parse {
    //kind: nom::error::ErrorKind,
  },
  #[error("Read error: {0}")]
  Read(#[from] T),
}

/// Detected ISO 9660 extension, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extension {
  /// Microsoft Joliet extension.
  Joliet,
  /// Rock Ridge extension.
  RockRidge,
}

/// Iterator over directory entries.
pub struct DirectoryIter<'a, Storage> {
  inner: Rc<spec::DirectoryRecord>,
  offset: u64,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> Iterator for DirectoryIter<'a, Storage>
where
  Storage: read::IsoRead,
{
  type Item = Result<DirectoryEntryRef<'a, Storage>, Storage::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    assert!(
      self.inner.is_directory(),
      "DirectoryIter can only be used on directories"
    );

    (|| {
      let mut sector = [0u8; 2048];

      let sector_ix = self.offset / 2048;
      let bytes_offset = self.offset % 2048;

      let read = self
        .iso
        .storage
        .borrow_mut()
        .read_sector(self.inner.extent_lba as u64 + sector_ix, &mut sector)?;

      // TODO(meowesque): This wont work for anything larger than 2048 bytes

      if bytes_offset >= read as u64 {
        return Ok(None);
      }

      let record_bytes = &sector[bytes_offset as usize..read as usize];

      let (remaining, record) = spec::DirectoryRecord::parse(record_bytes).map_err(|e| {
        dbg!(e);
        // TODO(meowesque): Handle
        Error::Parse {}
      })?;

      if record.record_length == 0 {
        // Padding, skip to next sector
        self.offset += 2048 - bytes_offset;
        return self.next().transpose();
      }

      match () {
        _ if record.identifier == "\u{0}" || record.identifier == "\u{1}" => {
          // Special entries, skip
          self.offset += record.record_length as u64;
          return self.next().transpose();
        }
        _ if record.file_flags.contains(spec::FileFlags::DIRECTORY) => {
          self.offset += record.record_length as u64;
          return Ok(Some(DirectoryEntryRef::Directory(DirectoryRef {
            inner: Rc::new(record),
            iso: self.iso,
          })));
        }
        _ if record.file_flags.is_empty() => {
          self.offset += record.record_length as u64;
          return Ok(Some(DirectoryEntryRef::File(FileRef {
            inner: Rc::new(record),
            iso: self.iso,
          })));
        }
        _ => todo!(),
      }

      Ok(None)
    })()
    .transpose()
  }
}

pub struct FileRef<'a, Storage> {
  inner: Rc<spec::DirectoryRecord>,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> FileRef<'a, Storage> {
  pub fn name(&self) -> &str {
    let name = self
      .inner
      .as_ref()
      .identifier
      .as_str()
      .trim_end_matches(char::is_numeric);

    &name[..name.len() - 1]
  }

  pub fn revision(&self) -> u8 {
    self
      .inner
      .as_ref()
      .identifier
      .as_str()
      .rsplit_once(';')
      .and_then(|(_, rev)| rev.parse().ok())
      .unwrap_or(1)
  }
}

pub enum DirectoryEntryRef<'a, Storage> {
  File(FileRef<'a, Storage>),
  Directory(DirectoryRef<'a, Storage>),
}

pub struct DirectoryRef<'a, Storage> {
  inner: Rc<spec::DirectoryRecord>,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> DirectoryRef<'a, Storage> {
  pub fn name(&self) -> impl AsRef<str> + '_ {
    self.inner.as_ref().identifier.as_str()
  }

  pub fn entries(&self) -> DirectoryIter<'a, Storage> {
    DirectoryIter {
      inner: self.inner.clone(),
      offset: 0,
      iso: self.iso,
    }
  }
}

pub struct PrimaryVolumeRef<'a, Storage> {
  inner: &'a spec::PrimaryVolumeDescriptor,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> PrimaryVolumeRef<'a, Storage> {
  pub fn identifier(&self) -> impl AsRef<str> + '_ {
    self.inner.volume_identifier.as_str()
  }

  /// Retrieve the volume descriptor.
  pub fn descriptor(&self) -> &spec::PrimaryVolumeDescriptor {
    self.inner
  }

  /// Retrieve the root directory of the volume.
  pub fn root(&self) -> DirectoryRef<'a, Storage> {
    DirectoryRef {
      inner: Rc::new(self.inner.root_directory_record.clone()),
      iso: self.iso,
    }
  }
}

pub struct SupplementaryVolumeRef<'a, Storage> {
  inner: &'a spec::SupplementaryVolumeDescriptor,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> SupplementaryVolumeRef<'a, Storage> {
  pub fn identifier(&self) -> impl AsRef<str> + '_ {
    self.inner.volume_identifier.as_str()
  }

  /// Retrieve the volume descriptor.
  pub fn descriptor(&self) -> &spec::SupplementaryVolumeDescriptor {
    self.inner
  }

  /// Retrieve the root directory of the volume.
  pub fn root(&self) -> DirectoryRef<'a, Storage> {
    DirectoryRef {
      inner: Rc::new(self.inner.root_directory_record.clone()),
      iso: self.iso,
    }
  }
}

pub enum VolumeRef<'a, Storage> {
  Primary(PrimaryVolumeRef<'a, Storage>),
  Supplementary(SupplementaryVolumeRef<'a, Storage>),
}

impl<'a, Storage> VolumeRef<'a, Storage> {
  pub fn descriptor(&self) -> spec::VolumeDescriptor {
    match self {
      VolumeRef::Primary(v) => spec::VolumeDescriptor::Primary(v.inner.clone()),
      VolumeRef::Supplementary(v) => spec::VolumeDescriptor::Supplementary(v.inner.clone()),
    }
  }
}

pub struct VolumesIter<'a, Storage> {
  inner: std::slice::Iter<'a, VolumeRef<'a, Storage>>,
}

pub struct Iso<Storage> {
  extension: Option<Extension>,
  storage: RefCell<Storage>,
  volume_descriptors: Vec<spec::VolumeDescriptor>,
}

impl<Storage> Iso<Storage>
where
  Storage: read::IsoRead,
{
  pub fn open(storage: Storage) -> Result<Self, Storage::Error> {
    // TODO(meowesque): Begin volume_descriptors discovery
    let mut iso = Self {
      extension: None,
      storage: RefCell::new(storage),
      volume_descriptors: vec![],
    };

    iso.scan()?;

    Ok(iso)
  }

  pub fn volumes<'a>(&'a self) -> impl Iterator<Item = VolumeRef<'a, Storage>> {
    self
      .volume_descriptors
      .iter()
      .map(|descriptor| match descriptor {
        spec::VolumeDescriptor::Primary(v) => VolumeRef::Primary(PrimaryVolumeRef {
          inner: v,
          iso: self,
        }),
        spec::VolumeDescriptor::Supplementary(v) => {
          VolumeRef::Supplementary(SupplementaryVolumeRef {
            inner: v,
            iso: self,
          })
        }
      })
  }

  pub fn scan(&mut self) -> Result<(), Storage::Error> {
    let mut sector = [0u8; 2048];
    let mut sector_ix = 0;
    let mut volumes = vec![];

    loop {
      let read = self
        .storage
        .borrow_mut()
        .read_sector(0x10 + sector_ix, &mut sector)?;

      match () {
        // End of input
        _ if read == 0 => break,
        // Unexpected short read
        _ if read < 2048 => {
          log::warn!("Unexpected short read ({}) expected 2048", read);
          break;
        }
        _ => {}
      }

      let (_, volume) = spec::VolumeDescriptor::parse(&sector).map_err(|e| {
        dbg!(e);
        // TODO(meowesque): Handle
        Error::Parse {}
      })?;

      match volume {
        Some(v) => volumes.push(v),
        None => break,
      }

      sector_ix += 1;
    }

    self.volume_descriptors = volumes;

    Ok(())
  }
}
