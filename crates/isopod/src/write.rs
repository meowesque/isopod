pub trait IsoWrite {
  type Error;

  fn write_at(&mut self, sector: u64, data: &[u8]) -> Result<usize, Self::Error>;
}

impl<T> IsoWrite for T
where
  T: std::io::Write + std::io::Seek,
{
  type Error = std::io::Error;

  fn write_at(&mut self, sector: u64, data: &[u8]) -> Result<usize, Self::Error> {
    use std::io::SeekFrom;

    const SECTOR_SIZE: u64 = 2048;

    self.seek(SeekFrom::Start(sector * SECTOR_SIZE))?;
    let written = self.write(&data[..])?;

    Ok(written)
  }
}