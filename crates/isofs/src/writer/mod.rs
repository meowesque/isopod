use crate::{serialize::IsoSerialize, spec, writer::volume::VolumeLike};

pub mod error;
pub mod fs;
pub mod lba;
pub mod sector;
pub mod volume;

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
