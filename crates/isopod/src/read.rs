

pub trait IsoRead {
  
}

impl<T> IsoRead for T where T: std::io::Read + std::io::Seek {
  
}

