use crate::spec;

pub trait IsoSerialize {
  type Error;

  fn serialize(&self, out: &mut [u8]) -> Result<usize, Self::Error>;
}

impl IsoSerialize for spec::IsoDateTime {
  type Error = &'static str;

  fn serialize(&self, out: &mut [u8]) -> Result<usize, Self::Error> {
    if out.len() < 7 {
      return Err("Output buffer too small");
    }

    out[0] = self.years_since_1900;
    out[1] = self.month;
    out[2] = self.day;
    out[3] = self.hour;
    out[4] = self.minute;
    out[5] = self.second;
    out[6] = self.offset as u8;

    Ok(7)
  }
}

impl IsoSerialize for spec::DirectoryRecord {
  type Error = &'static str;

  fn serialize(&self, out: &mut [u8]) -> Result<usize, Self::Error> {
    // TODO(meowesque): Check if this is correct.

    let record_length = self.record_length() as usize;
    if out.len() < record_length {
      return Err("Output buffer too small");
    }

    out[0] = self.record_length();
    out[1] = self.extended_attribute_record_length;
    out[2..6].copy_from_slice(&self.extent_lba.to_le_bytes());
    out[6..10].copy_from_slice(&self.extent_length.to_le_bytes());
    self.recording_date.serialize(&mut out[10..17])?;
    out[17] = self.file_flags.bits();
    out[18] = self.file_unit_size;
    out[19] = self.interleave_gap_size;
    out[20..24].copy_from_slice(&self.volume_sequence_number.to_le_bytes());
    out[24] = self.identifier_length;
    out[25..25 + self.identifier_length as usize].copy_from_slice(self.identifier.as_bytes());

    if self.record_length() % 2 == 1 {
      out[25 + self.identifier_length as usize] = 0;
    }

    Ok(record_length)
  }
}
