pub(crate) struct LbaAllocator {
  sector_size: u32,
  next_lba: u32,
}

impl LbaAllocator {
  pub(crate) fn new(sector_size: u32, start_lba: u32) -> Self {
    Self {
      sector_size,
      next_lba: start_lba,
    }
  }

  pub(crate) fn allocate(&mut self, size: u32) -> u32 {
    let lba = self.next_lba;
    let sectors = (size + self.sector_size - 1) / self.sector_size;
    self.next_lba += sectors;
    lba
  }
}