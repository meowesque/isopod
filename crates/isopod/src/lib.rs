// ISO 9660 Library implementation
//
// This library provides functionality for creating, reading, and manipulating
// ISO 9660 filesystem images.

mod error;
mod iso;
mod volume;
mod directory;
mod file;
mod utils;

pub use error::Error;
pub use iso::{Iso, IsoBuilder};
pub use volume::{VolumeDescriptor, PrimaryVolumeDescriptor};
pub use directory::{Directory, DirectoryEntry};
pub use file::File;

/// Result type for operations that may return an Error
pub type Result<T> = std::result::Result<T, Error>;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// ISO 9660 standard constants
pub mod constants {
    /// Standard identifier for ISO 9660
    pub const ISO_STANDARD_ID: &[u8; 5] = b"CD001";
    
    /// Volume descriptor types
    pub mod volume_type {
        pub const BOOT_RECORD: u8 = 0;
        pub const PRIMARY_VOLUME_DESCRIPTOR: u8 = 1;
        pub const SUPPLEMENTARY_VOLUME_DESCRIPTOR: u8 = 2;
        pub const VOLUME_PARTITION_DESCRIPTOR: u8 = 3;
        pub const VOLUME_DESCRIPTOR_SET_TERMINATOR: u8 = 255;
    }
    
    /// Sector size (2048 bytes)
    pub const SECTOR_SIZE: usize = 2048;
    
    /// Maximum filename length in ISO 9660 Level 1
    pub const MAX_FILENAME_LENGTH_LEVEL_1: usize = 8;
    
    /// Maximum extension length in ISO 9660 Level 1
    pub const MAX_EXTENSION_LENGTH_LEVEL_1: usize = 3;
    
    /// Maximum path depth in ISO 9660
    pub const MAX_PATH_DEPTH: usize = 8;
}