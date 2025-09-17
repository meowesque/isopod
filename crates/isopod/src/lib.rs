pub mod parse;
pub mod read;
pub mod spec;
pub mod write;

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

bitflags::bitflags! {
  /// Supported extensions.
  pub struct Extensions: u8 {
    /// No extensions.
    const NONE = 0;
    /// Rock Ridge extensions.
    const ROCK_RIDGE = 1 << 0;
    /// Joliet extensions.
    const JOLIET = 1 << 1;
  }
}

impl Default for Extensions {
  fn default() -> Self {
    Self::NONE
  }
}

/// Iterator over directory entries.
pub struct DirectoryIter<'a, Storage> {
  valid: bool,
  inner: spec::DirectoryRecord,
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

    // TODO(meowesque): Handle invalid state more gracefully
    if !self.valid {
      return None;
    }

    let sector_ix = self.offset / 2048;
    let bytes_offset = self.offset % 2048;

    if self.offset >= self.inner.extent_length as u64 {
      // End of directory records
      return None;
    }

    let mut sector = [0u8; 2048];

    let read = match self
      .iso
      .storage
      .borrow_mut()
      .read_sector(self.inner.extent_lba as u64 + sector_ix, &mut sector)
    {
      Ok(0) => {
        log::warn!(
          "Unexpected end of input, expected atleast {} more bytes",
          self.inner.extent_length - self.offset as u32
        );
        return None;
      }
      Ok(read) => read,
      Err(e) => return Some(Err(Error::Read(e))),
    };

    let record_bytes = &sector[bytes_offset as usize..read as usize];

    if record_bytes[0] == 0 {
      // The first byte being zero indicates padding at the end of the sector.
      self.offset += 2048 - bytes_offset;
      return self.next();
    }

    let record = match spec::DirectoryRecord::parse(record_bytes) {
      Ok((_, record)) => record,
      Err(e) => {
        // TODO(meowesque): Handle parse error
        return Some(Err(Error::Parse {
          //kind: e.code,
        }));
      }
    };

    self.offset += record.record_length as u64;

    match () {
      // TODO(meowesque): Ignore this or return it?
      _ if record.identifier == "\u{0}" || record.identifier == "\u{1}" => {
        // Special entries, skip
        self.next()
      }
      // Directory
      _ if record.file_flags.contains(spec::FileFlags::DIRECTORY) => {
        Some(Ok(DirectoryEntryRef::Directory(DirectoryRef {
          inner: record,
          iso: self.iso,
        })))
      }
      // Regular file
      _ if record.file_flags.is_empty() => Some(Ok(DirectoryEntryRef::File(FileRef {
        inner: record,
        iso: self.iso,
      }))),
      // TODO(meowesque): Handle other types of entries
      _ => todo!(),
    }
  }
}

pub struct FileRef<'a, Storage> {
  inner: spec::DirectoryRecord,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> FileRef<'a, Storage> {
  pub fn name(&self) -> &str {
    let name = self
      .inner
      .identifier
      .as_str()
      .trim_end_matches(char::is_numeric);

    &name[..name.len() - 1]
  }

  pub fn revision(&self) -> u8 {
    self
      .inner
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
  inner: spec::DirectoryRecord,
  iso: &'a Iso<Storage>,
}

impl<'a, Storage> DirectoryRef<'a, Storage> {
  pub fn name(&self) -> &str {
    self.inner.identifier.as_str()
  }

  pub fn entries(&self) -> DirectoryIter<'a, Storage> {
    DirectoryIter {
      valid: true,
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
      inner: self.inner.root_directory_record.clone(),
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
      inner: self.inner.root_directory_record.clone(),
      iso: self.iso,
    }
  }
}

pub struct Iso<Storage> {
  extensions: Extensions,
  storage: RefCell<Storage>,
  pvd: Option<spec::PrimaryVolumeDescriptor>,
  svd: Option<spec::SupplementaryVolumeDescriptor>,
}

impl<Storage> Iso<Storage>
where
  Storage: read::IsoRead,
{
  /// Open an ISO image from the given storage backend, with the specified extensions enabled.
  pub fn open(storage: Storage, extensions: Extensions) -> Result<Self, Storage::Error> {
    let mut iso = Self {
      extensions,
      storage: RefCell::new(storage),
      pvd: None,
      svd: None,
    };

    iso.scan()?;

    Ok(iso)
  }

  /// Retrieve the Primary Volume Descriptor, if present.
  pub fn primary_volume(&self) -> Option<PrimaryVolumeRef<'_, Storage>> {
    self
      .pvd
      .as_ref()
      .map(|inner| PrimaryVolumeRef { inner, iso: self })
  }

  /// Retrieve the Supplementary Volume Descriptor, if present.
  pub fn supplementary_volume(&self) -> Option<SupplementaryVolumeRef<'_, Storage>> {
    self
      .svd
      .as_ref()
      .map(|inner| SupplementaryVolumeRef { inner, iso: self })
  }

  fn scan(&mut self) -> Result<(), Storage::Error> {
    let mut sector = [0u8; 2048];
    let mut sector_ix = 0;

    loop {
      let read = self
        .storage
        .borrow_mut()
        .read_sector(spec::STARTING_SECTOR + sector_ix, &mut sector)?;

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

      let (_, vd) = spec::VolumeDescriptor::parse(&sector).map_err(|e| {
        // TODO(meowesque): Handle
        Error::Parse {}
      })?;

      let Some(vd) = vd else {
        // Set terminator encountered.
        break;
      };

      match vd {
        spec::VolumeDescriptor::Primary(pvd) if self.pvd.is_none() => self.pvd = Some(pvd),
        spec::VolumeDescriptor::Supplementary(svd)
          if self.svd.is_none() && self.extensions.contains(Extensions::JOLIET) =>
        {
          // Only accept the first SVD if Joliet extensions are enabled
          self.svd = Some(svd)
        }

        // Ignore unsupported descriptors
        spec::VolumeDescriptor::Supplementary(_)
          if !self.extensions.contains(Extensions::JOLIET) =>
        {
          // TODO(meowesque): Display more information about the ignored volume descriptor
          log::warn!("Joliet extensions not enabled, ignoring Supplementary Volume Descriptor");
        }

        // Ignore duplicates
        spec::VolumeDescriptor::Primary(_) => {
          // TODO(meowesque): Display more information about the ignored volume descriptor
          log::warn!("Multiple Primary Volume Descriptors found, ignoring subsequent ones");
        }
        spec::VolumeDescriptor::Supplementary(_) => {
          // TODO(meowesque): Display more information about the ignored volume descriptor
          log::warn!("Multiple Supplementary Volume Descriptors found, ignoring subsequent ones");
        }
      }

      sector_ix += 1;
    }

    Ok(())
  }
}
