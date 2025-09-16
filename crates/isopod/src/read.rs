use crate::spec;

pub trait IsoRead {
  type Error;

  fn read_sector(&mut self, sector: u64, out: &mut [u8]) -> Result<usize, Self::Error>;
}

impl<T> IsoRead for T
where
  T: std::io::Read + std::io::Seek,
{
  type Error = std::io::Error;

  fn read_sector(&mut self, sector: u64, out: &mut [u8]) -> Result<usize, Self::Error> {
    use std::io::SeekFrom;

    // TODO(meowesque): Handle different sector sizes
    const SECTOR_SIZE: u64 = 2048;

    self.seek(SeekFrom::Start((sector) * SECTOR_SIZE))?;
    let read = self.read(&mut out[..SECTOR_SIZE as usize])?;

    Ok(read)
  }
}
