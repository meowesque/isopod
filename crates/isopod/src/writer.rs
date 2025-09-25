#[derive(Debug)]
pub struct WriterOptions {
  
}

pub struct IsoWriter {
  options: WriterOptions,
}

impl IsoWriter {
  pub fn new(options: WriterOptions) -> Self {
    Self { options }
  }
}