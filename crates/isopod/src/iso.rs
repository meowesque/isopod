use log::debug;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::constants::SECTOR_SIZE;
use crate::directory::{Directory, DirectoryEntry};
use crate::volume::{PrimaryVolumeDescriptor, VolumeDescriptor};
use crate::{Error, Result};

/// Represents an ISO 9660 image
pub struct Iso {
  /// The primary volume descriptor for this ISO
  volume_descriptor: PrimaryVolumeDescriptor,

  /// Root directory of the ISO filesystem
  root_directory: Directory,

  /// Path to the backing file if loaded from or saved to disk
  path: Option<PathBuf>,

  /// Whether this ISO has been modified since loading
  modified: bool,

  /// Extensions supported by this ISO
  extensions: IsoExtensions,
}

/// Supported ISO 9660 extensions
#[derive(Debug, Default, Clone)]
pub struct IsoExtensions {
  /// Joliet extension for long file names and Unicode support
  pub joliet: bool,

  /// Rock Ridge extension for POSIX filesystem features
  pub rock_ridge: bool,

  /// El Torito extension for bootable CDs
  pub el_torito: bool,

  /// UDF (Universal Disk Format) bridge
  pub udf: bool,
}

impl Iso {
  /// Create a new builder for an ISO
  pub fn builder() -> IsoBuilder {
    IsoBuilder::new()
  }

  /// Open an existing ISO file
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
    let mut file = File::open(path.as_ref())?;
    let mut iso = Self::read_from(&mut file)?;
    iso.path = Some(path.as_ref().to_path_buf());
    Ok(iso)
  }

  /// Read an ISO from any reader
  pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    // Read the first 16 sectors to find volume descriptors
    let sector_count = 16;
    let buffer_size = SECTOR_SIZE * sector_count;
    let mut buffer = vec![0u8; buffer_size];

    // Seek to the beginning of the file
    reader.seek(SeekFrom::Start(0))?;

    // Try to read the full buffer, but handle if file is smaller
    match reader.read_exact(&mut buffer) {
      Ok(_) => {}
      Err(e) => {
        if e.kind() != std::io::ErrorKind::UnexpectedEof {
          return Err(Error::Io(e));
        }

        // If we got an unexpected EOF, continue with what we read
        debug!("Warning: ISO appears to be smaller than expected. Continuing with partial data.");
      }
    }

    // Find and parse the primary volume descriptor
    let volume_descriptor = PrimaryVolumeDescriptor::parse_from_buffer(&buffer)
      .ok_or_else(|| Error::InvalidFormat("Failed to find primary volume descriptor".into()))?;

    // Parse root directory
    let root_entry = volume_descriptor.root_directory_entry().clone();

    // Validate root entry
    if root_entry.extent_location() == 0 || root_entry.extent_size() == 0 {
      return Err(Error::InvalidFormat("Invalid root directory entry".into()));
    }

    // Read the root directory
    let root_directory = match Directory::read_from_iso(reader, &root_entry) {
      Ok(dir) => dir,
      Err(e) => {
        return Err(Error::InvalidFormat(format!(
          "Failed to read root directory: {}",
          e
        )));
      }
    };

    // Detect extensions
    let extensions = Self::detect_extensions(reader, &buffer)?;

    Ok(Self {
      volume_descriptor,
      root_directory,
      path: None,
      modified: false,
      extensions,
    })
  }

  /// Save the ISO to a file
  pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
    let mut file = File::create(path.as_ref())?;
    self.write_to(&mut file)?;
    self.path = Some(path.as_ref().to_path_buf());
    self.modified = false;
    Ok(())
  }

  // In the Iso implementation, modify the write_to method
  pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    // Debug: Log initial file position
    let initial_pos = writer.stream_position()?;
    println!("Starting write at position: {}", initial_pos);

    // First 16 sectors are reserved for system area
    let system_area = vec![0u8; SECTOR_SIZE * 16];
    writer.write_all(&system_area)?;

    // Verify position after system area
    let pos_after_system_area = writer.stream_position()?;
    println!("Position after system area: {}", pos_after_system_area);

    // Write primary volume descriptor
    self.volume_descriptor.write_to(writer)?;

    // Debug: Verify volume descriptor was written
    let pos_after_volume_descriptor = writer.stream_position()?;
    println!(
      "Position after volume descriptor: {}",
      pos_after_volume_descriptor
    );

    // Write volume descriptor set terminator
    self.write_volume_descriptor_set_terminator(writer)?;

    // Verify terminator was written
    let pos_after_terminator = writer.stream_position()?;
    println!("Position after terminator: {}", pos_after_terminator);

    // Write directory and file data
    self.root_directory.write_to_iso(writer)?;

    // Finalize the ISO
    writer.flush()?;

    Ok(())
  }

  /// Add a file to the ISO
  pub fn add_file<P: AsRef<Path>, S: AsRef<Path>>(
    &mut self,
    iso_path: P,
    source_path: S,
  ) -> Result<()> {
    // Load the file content
    let mut file = File::open(source_path.as_ref())?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // Add to the ISO
    self.add_file_with_content(iso_path, &content)?;

    self.modified = true;
    Ok(())
  }

  /// Add a file with the provided content
  pub fn add_file_with_content<P: AsRef<Path>>(
    &mut self,
    iso_path: P,
    content: &[u8],
  ) -> Result<()> {
    let path = iso_path.as_ref();

    // Validate the path
    self.validate_path(path)?;

    // Get parent directory path
    let parent_path = path.parent().unwrap_or_else(|| Path::new(""));

    // Get or create parent directory
    let parent_dir = self.get_or_create_directory(parent_path)?;

    // Create file entry
    let filename = path
      .file_name()
      .ok_or_else(|| Error::PathError("Invalid filename".into()))?
      .to_string_lossy()
      .into_owned();

    // Add file to parent directory
    parent_dir.add_file(filename, content)?;

    self.modified = true;
    Ok(())
  }

  /// Add a directory to the ISO
  pub fn add_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
    let path = path.as_ref();

    // Validate the path
    self.validate_path(path)?;

    // Get or create the directory
    self.get_or_create_directory(path)?;

    self.modified = true;
    Ok(())
  }

  /// Extract a file from the ISO
  pub fn extract_file<P: AsRef<Path>, D: AsRef<Path>>(
    &self,
    iso_path: P,
    dest_path: D,
  ) -> Result<()> {
    let file_content = self.get_file_content(iso_path)?;

    // Write to destination
    let mut file = File::create(dest_path.as_ref())?;
    file.write_all(&file_content)?;

    Ok(())
  }

  /// Get a file's content from the ISO
  pub fn get_file_content<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
    let path = path.as_ref();

    // Navigate to file location
    let parent_path = path.parent().unwrap_or_else(|| Path::new(""));
    let filename = path
      .file_name()
      .ok_or_else(|| Error::PathError("Invalid filename".into()))?
      .to_string_lossy();

    let directory = self.find_directory(parent_path)?;
    let file_entry = directory
      .find_file(&filename)
      .ok_or_else(|| Error::PathError(format!("File not found: {}", path.display())))?;

    Ok(file_entry.content().to_vec())
  }

  /// Get the root directory
  pub fn root_directory(&self) -> &Directory {
    &self.root_directory
  }

  /// Get a mutable reference to the root directory
  pub fn root_directory_mut(&mut self) -> &mut Directory {
    self.modified = true;
    &mut self.root_directory
  }

  /// Get the volume descriptor
  pub fn volume_descriptor(&self) -> &PrimaryVolumeDescriptor {
    &self.volume_descriptor
  }

  /// Get a mutable reference to the volume descriptor
  pub fn volume_descriptor_mut(&mut self) -> &mut PrimaryVolumeDescriptor {
    self.modified = true;
    &mut self.volume_descriptor
  }

  /// Check if the ISO has been modified
  pub fn is_modified(&self) -> bool {
    self.modified
  }

  /// Get the ISO extensions
  pub fn extensions(&self) -> &IsoExtensions {
    &self.extensions
  }

  // Private helper methods

  /// Validate a path for ISO 9660 compatibility
  fn validate_path(&self, path: &Path) -> Result<()> {
    // Check path depth
    if path.components().count() > crate::constants::MAX_PATH_DEPTH {
      return Err(Error::PathError(format!(
        "Path depth exceeds maximum of {}: {}",
        crate::constants::MAX_PATH_DEPTH,
        path.display()
      )));
    }

    // Skip validation for Joliet/Rock Ridge if enabled
    if self.extensions.joliet || self.extensions.rock_ridge {
      return Ok(());
    }

    // For each component, validate the filename
    for component in path.components() {
      if let std::path::Component::Normal(name) = component {
        let filename = name.to_string_lossy();

        // Check for ISO 9660 Level 1 compliance
        if let Some((name, ext)) = filename.split_once('.') {
          if name.len() > crate::constants::MAX_FILENAME_LENGTH_LEVEL_1 {
            return Err(Error::PathError(format!(
              "Filename '{}' exceeds maximum length of {}",
              name,
              crate::constants::MAX_FILENAME_LENGTH_LEVEL_1
            )));
          }

          if ext.len() > crate::constants::MAX_EXTENSION_LENGTH_LEVEL_1 {
            return Err(Error::PathError(format!(
              "Extension '{}' exceeds maximum length of {}",
              ext,
              crate::constants::MAX_EXTENSION_LENGTH_LEVEL_1
            )));
          }
        } else if filename.len() > crate::constants::MAX_FILENAME_LENGTH_LEVEL_1 {
          return Err(Error::PathError(format!(
            "Filename '{}' exceeds maximum length of {}",
            filename,
            crate::constants::MAX_FILENAME_LENGTH_LEVEL_1
          )));
        }

        // Check for valid characters
        if !filename
          .chars()
          .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
          return Err(Error::PathError(format!("Invalid filename")));
        }
      }
    }

    Ok(())
  }

  /// Find a directory by path
  fn find_directory(&self, path: &Path) -> Result<&Directory> {
    if path.as_os_str().is_empty() {
      return Ok(&self.root_directory);
    }

    let mut current_dir = &self.root_directory;

    for component in path.components() {
      if let std::path::Component::Normal(name) = component {
        let dir_name = name.to_string_lossy();
        current_dir = current_dir
          .find_directory(&dir_name)
          .ok_or_else(|| Error::PathError(format!("Directory not found: {}", dir_name)))?;
      }
    }

    Ok(current_dir)
  }

  /// Get or create a directory by path
  fn get_or_create_directory(&mut self, path: &Path) -> Result<&mut Directory> {
    if path.as_os_str().is_empty() {
      return Ok(&mut self.root_directory);
    }

    let mut current_dir = &mut self.root_directory;

    for component in path.components() {
      if let std::path::Component::Normal(name) = component {
        let dir_name = name.to_string_lossy().into_owned();

        // Check if directory exists
        if current_dir.find_directory(&dir_name).is_none() {
          // Create if it doesn't
          current_dir.add_directory(dir_name.clone())?;
        }

        // Now it definitely exists, get a mutable reference to it
        current_dir = current_dir
          .find_directory_mut(&dir_name)
          .ok_or_else(|| Error::PathError(format!("Failed to create directory: {}", dir_name)))?;
      }
    }

    Ok(current_dir)
  }

  /// Write volume descriptor set terminator
  fn write_volume_descriptor_set_terminator<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    let sector_position = SECTOR_SIZE as u64 * (16 + 1); // After primary volume descriptor
    writer.seek(SeekFrom::Start(sector_position))?;

    let mut terminator = [0u8; SECTOR_SIZE];

    // Type code
    terminator[0] = crate::constants::volume_type::VOLUME_DESCRIPTOR_SET_TERMINATOR;

    // Standard identifier
    terminator[1..6].copy_from_slice(crate::constants::ISO_STANDARD_ID);

    // Version
    terminator[6] = 1;

    writer.write_all(&terminator)?;

    Ok(())
  }

  /// Detect ISO extensions
  fn detect_extensions<R: Read + Seek>(reader: &mut R, buffer: &[u8]) -> Result<IsoExtensions> {
    let mut extensions = IsoExtensions::default();

    // Scan for Joliet Supplementary Volume Descriptor
    for sector in 0..16 {
      let offset = sector * SECTOR_SIZE;
      if offset + 7 <= buffer.len() {
        if buffer[offset] == crate::constants::volume_type::SUPPLEMENTARY_VOLUME_DESCRIPTOR
          && &buffer[offset + 1..offset + 6] == crate::constants::ISO_STANDARD_ID
        {
          // Check escape sequences for Joliet
          if offset + 88 <= buffer.len() {
            let escape_seq = &buffer[offset + 88..offset + 88 + 3];
            if escape_seq == b"%/E" || escape_seq == b"%/C" || escape_seq == b"%/G" {
              extensions.joliet = true;
            }
          }
        }
      }
    }

    // TODO: Implement detection for Rock Ridge, El Torito, and UDF

    Ok(extensions)
  }
}

/// Builder for creating ISO images
pub struct IsoBuilder {
  volume_id: String,
  publisher: String,
  preparer: String,
  application: String,
  extensions: IsoExtensions,
}

impl IsoBuilder {
  /// Create a new ISO builder
  pub fn new() -> Self {
    Self {
      volume_id: "ISO_VOLUME".to_string(),
      publisher: "".to_string(),
      preparer: "isopod".to_string(),
      application: format!("isopod {}", crate::VERSION),
      extensions: IsoExtensions::default(),
    }
  }

  /// Set the volume ID
  pub fn volume_id<S: Into<String>>(mut self, volume_id: S) -> Self {
    self.volume_id = volume_id.into();
    self
  }

  /// Set the publisher
  pub fn publisher<S: Into<String>>(mut self, publisher: S) -> Self {
    self.publisher = publisher.into();
    self
  }

  /// Set the data preparer
  pub fn preparer<S: Into<String>>(mut self, preparer: S) -> Self {
    self.preparer = preparer.into();
    self
  }

  /// Set the application
  pub fn application<S: Into<String>>(mut self, application: S) -> Self {
    self.application = application.into();
    self
  }

  /// Enable Joliet extension
  pub fn joliet(mut self, enable: bool) -> Self {
    self.extensions.joliet = enable;
    self
  }

  /// Enable Rock Ridge extension
  pub fn rock_ridge(mut self, enable: bool) -> Self {
    self.extensions.rock_ridge = enable;
    self
  }

  /// Enable El Torito extension
  pub fn el_torito(mut self, enable: bool) -> Self {
    self.extensions.el_torito = enable;
    self
  }

  /// Enable UDF bridge
  pub fn udf(mut self, enable: bool) -> Self {
    self.extensions.udf = enable;
    self
  }

  /// Build the ISO image
  pub fn build(self) -> Result<Iso> {
    // Create root directory
    let root_directory = Directory::new("ROOT");

    // Create primary volume descriptor
    let volume_descriptor = PrimaryVolumeDescriptor::new(
      &self.volume_id,
      &self.publisher,
      &self.preparer,
      &self.application,
      &root_directory,
    );

    Ok(Iso {
      volume_descriptor,
      root_directory,
      path: None,
      modified: true,
      extensions: self.extensions,
    })
  }
}

impl Default for IsoBuilder {
  fn default() -> Self {
    Self::new()
  }
}
