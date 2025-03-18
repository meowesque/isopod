use std::collections::HashMap;
use std::io::{Read, Write, Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::SECTOR_SIZE;
use crate::file::File;
use crate::Error;
use crate::Result;
use crate::utils;

/// Directory entry flags
#[repr(u8)]
pub enum DirectoryEntryFlag {
    Hidden = 0x01,
    Directory = 0x02,
    AssociatedFile = 0x04,
    Record = 0x08,
    Protection = 0x10,
    MultiExtent = 0x80,
}

/// Represents a directory entry in the ISO 9660 filesystem
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// Entry name
    name: String,
    
    /// Entry flags
    flags: u8,
    
    /// Location of extent (sector)
    extent_location: u32,
    
    /// Size of extent (bytes)
    extent_size: u32,
    
    /// Recording date and time
    recording_time: SystemTime,
}

impl DirectoryEntry {
    /// Create a new file entry
    pub fn new_file(name: &str, extent_location: u32, extent_size: u32) -> Self {
        Self {
            name: name.to_string(),
            flags: 0,
            extent_location,
            extent_size,
            recording_time: SystemTime::now(),
        }
    }
    
    /// Create a new directory entry
    pub fn new_directory(name: &str, extent_location: u32, extent_size: u32) -> Self {
        Self {
            name: name.to_string(),
            flags: DirectoryEntryFlag::Directory as u8,
            extent_location,
            extent_size,
            recording_time: SystemTime::now(),
        }
    }
    
    /// Parse a directory entry from a buffer
    pub fn parse_from_buffer(buffer: &[u8]) -> Option<Self> {
        // Buffer needs to be at least 33 bytes to contain the minimum directory record
        if buffer.len() < 33 || buffer[0] == 0 {
            return None;
        }
        
        // Record length
        let record_length = buffer[0] as usize;
        if record_length < 33 || record_length > buffer.len() {
            return None;
        }
        
        // Extended attribute record length
        let _ext_attr_length = buffer[1];
        
        // Extent location
        let extent_location = utils::parse_u32_both(&buffer[2..10]);
        
        // Data length
        let extent_size = utils::parse_u32_both(&buffer[10..18]);
        
        // Recording date/time
        let recording_time = utils::parse_recording_date(&buffer[18..25])
            .unwrap_or(UNIX_EPOCH);
        
        // File flags
        let flags = buffer[25];
        
        // File unit size and interleave
        let _file_unit_size = buffer[26];
        let _interleave_gap = buffer[27];
        
        // Volume sequence number
        let _volume_sequence = utils::parse_u16_both(&buffer[28..32]);
        
        // File identifier length
        let file_id_length = buffer[32] as usize;
        
        if 33 + file_id_length > record_length {
            return None;
        }
        
        // File identifier
        let name = if (flags & (DirectoryEntryFlag::Directory as u8)) != 0 && file_id_length == 1 {
            match buffer[33] {
                0 => ".".to_string(),   // Current directory
                1 => "..".to_string(),  // Parent directory
                _ => format!("UNKNOWN_{}", buffer[33]),
            }
        } else {
            let raw_name = &buffer[33..33 + file_id_length];
            // Detect if this is a file with a version suffix (;1)
            if raw_name.ends_with(b";1") {
                utils::parse_iso_string(&raw_name[0..raw_name.len() - 2])
            } else {
                utils::parse_iso_string(raw_name)
            }
        };
        
        Some(Self {
            name,
            flags,
            extent_location,
            extent_size,
            recording_time,
        })
    }
    
    /// Write the directory entry to a buffer
    pub fn write_to_buffer(&self, buffer: &mut [u8]) -> Result<()> {
        // Start with the base record length (33) + file ID length
        let file_id_bytes = self.iso_file_identifier();
        let file_id_length = file_id_bytes.len();
        let record_length = 33 + file_id_length;
        
        // Pad to even length if needed
        let padded_length = if record_length % 2 == 1 { record_length + 1 } else { record_length };
        
        if buffer.len() < padded_length {
            return Err(Error::InvalidFormat(format!(
                "Buffer too small for directory entry: need {} bytes, got {}",
                padded_length, buffer.len()
            )));
        }
        
        // Clear the buffer
        buffer[0..padded_length].fill(0);
        
        // Record length
        buffer[0] = padded_length as u8;
        
        // Extended attribute record length
        buffer[1] = 0;
        
        // Extent location (both little and big endian)
        utils::write_u32_both(&mut buffer[2..10], self.extent_location);
        
        // Data length (both little and big endian)
        utils::write_u32_both(&mut buffer[10..18], self.extent_size);
        
        // Recording date/time
        utils::write_recording_date(&mut buffer[18..25], self.recording_time);
        
        // File flags
        buffer[25] = self.flags;
        
        // File unit size and interleave
        buffer[26] = 0; // File unit size
        buffer[27] = 0; // Interleave gap
        
        // Volume sequence number
        utils::write_u16_both(&mut buffer[28..32], 1);
        
        // File identifier length
        buffer[32] = file_id_length as u8;
        
        // File identifier
        buffer[33..33 + file_id_length].copy_from_slice(&file_id_bytes);
        
        Ok(())
    }
    
    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the flags
    pub fn flags(&self) -> u8 {
        self.flags
    }
    
    /// Set the flags
    pub fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }
    
    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        (self.flags & (DirectoryEntryFlag::Directory as u8)) != 0
    }
    
    /// Get the extent location
    pub fn extent_location(&self) -> u32 {
        self.extent_location
    }
    
    /// Set the extent location
    pub fn set_extent_location(&mut self, location: u32) {
        self.extent_location = location;
    }
    
    /// Get the extent size
    pub fn extent_size(&self) -> u32 {
        self.extent_size
    }
    
    /// Set the extent size
    pub fn set_extent_size(&mut self, size: u32) {
        self.extent_size = size;
    }
    
    /// Get the recording time
    pub fn recording_time(&self) -> SystemTime {
        self.recording_time
    }
    
    /// Set the recording time
    pub fn set_recording_time(&mut self, time: SystemTime) {
        self.recording_time = time;
    }
    
    /// Calculate the record size of this entry
    pub fn record_size(&self) -> usize {
        let file_id_length = self.iso_file_identifier().len();
        let record_length = 33 + file_id_length;
        
        // Pad to even length if needed
        if record_length % 2 == 1 { record_length + 1 } else { record_length }
    }
    
    /// Get the ISO file identifier
    fn iso_file_identifier(&self) -> Vec<u8> {
        if self.is_directory() {
            if self.name == "." {
                return vec![0];
            } else if self.name == ".." {
                return vec![1];
            }
        }
        
        // Sanitize the name for ISO 9660 compliance (allow only A-Z, 0-9, _)
        let sanitized_name = self.name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
            
        // For files, append ";1" version
        if !self.is_directory() {
            let mut file_id = sanitized_name.as_bytes().to_vec();
            file_id.extend_from_slice(b";1");
            file_id
        } else {
            sanitized_name.as_bytes().to_vec()
        }
    }
}

/// Represents a directory in the ISO 9660 filesystem
#[derive(Debug, Clone)]
pub struct Directory {
    /// Directory name
    name: String,
    
    /// Subdirectories
    directories: HashMap<String, Directory>,
    
    /// Files
    files: HashMap<String, File>,
    
    /// Sector location
    sector_location: Option<u32>,
    
    /// Size in bytes
    size: Option<u32>,
}

impl Directory {
    /// Create a new directory
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            directories: HashMap::new(),
            files: HashMap::new(),
            sector_location: None,
            size: None,
        }
    }
    
    /// Read a directory from an ISO
    pub fn read_from_iso<R: Read + Seek>(reader: &mut R, entry: &DirectoryEntry) -> Result<Self> {
        if !entry.is_directory() {
            return Err(Error::InvalidFormat("Entry is not a directory".into()));
        }
        
        let location = entry.extent_location();
        let size = entry.extent_size();
        
        if location == 0 || size == 0 {
            return Err(Error::InvalidFormat(format!(
                "Invalid directory location ({}) or size ({})",
                location, size
            )));
        }
        
        // Sanity check the size to prevent excessive memory allocation
        if size > 10 * 1024 * 1024 { // Limit to 10MB for a directory
            return Err(Error::SizeLimit(format!(
                "Directory size exceeds reasonable limit: {} bytes",
                size
            )));
        }
        
        let mut buffer = vec![0u8; size as usize];
        let position = location as u64 * SECTOR_SIZE as u64;
        
        // Attempt to seek and read
        match reader.seek(SeekFrom::Start(position)) {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::Io(e));
            }
        }
        
        match reader.read_exact(&mut buffer) {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::Io(e));
            }
        }
        
        let mut directory = Self::new(entry.name());
        directory.sector_location = Some(location);
        directory.size = Some(size);
        
        // Parse directory entries
        let mut offset = 0;
        while offset < buffer.len() {
            // Check for end of entries
            if buffer[offset] == 0 || offset + 1 >= buffer.len() {
                break;
            }
            
            // Get record length
            let record_length = buffer[offset] as usize;
            if record_length == 0 || offset + record_length > buffer.len() {
                // Invalid record length, skip to next sector
                offset = (offset + SECTOR_SIZE) & !(SECTOR_SIZE - 1);
                continue;
            }
            
            // Parse directory entry
            if let Some(entry) = DirectoryEntry::parse_from_buffer(&buffer[offset..offset + record_length]) {
                // Skip . and .. entries
                if entry.name() != "." && entry.name() != ".." {
                    if entry.is_directory() {
                        // Recursively read subdirectory, but with safeguards against circular references
                        if entry.extent_location() != location { // Avoid self-referential directories
                            match Self::read_from_iso(reader, &entry) {
                                Ok(subdir) => {
                                    directory.directories.insert(entry.name().to_string(), subdir);
                                },
                                Err(e) => {
                                    // Log error but continue with other entries
                                    eprintln!("Error reading subdirectory '{}': {}", entry.name(), e);
                                }
                            }
                        }
                    } else {
                        // Read file content with safeguards
                        if entry.extent_location() > 0 && entry.extent_size() > 0 {
                            // Sanity check file size
                            if entry.extent_size() > 100 * 1024 * 1024 { // Limit to 100MB for a file
                                eprintln!("Skipping file '{}': size too large ({})", entry.name(), entry.extent_size());
                            } else {
                                let mut file_buffer = vec![0u8; entry.extent_size() as usize];
                                if let Ok(_) = reader.seek(SeekFrom::Start(entry.extent_location() as u64 * SECTOR_SIZE as u64)) {
                                    if let Ok(_) = reader.read_exact(&mut file_buffer) {
                                        // Create file
                                        let file = File::new(entry.name(), file_buffer);
                                        directory.files.insert(entry.name().to_string(), file);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            offset += record_length;
        }
        
        Ok(directory)
    }
    
    /// Write the directory to an ISO
    pub fn write_to_iso<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // First, write all subdirectories and files to get their locations
        let mut updated_directories = HashMap::new();
        let mut updated_files = HashMap::new();
        
        // Start after the system area, volume descriptors, and path tables
        // This is just an example - in a real implementation this would be tracked
        let mut next_sector = 20;
        
        // Write subdirectories
        for (name, directory) in &self.directories {
            // Create a copy of the directory with updated sector information
            let mut updated_dir = directory.clone();
            updated_dir.sector_location = Some(next_sector);
            
            // Calculate how many sectors this directory will occupy
            let dir_size = updated_dir.calculate_size();
            let sectors_needed = (dir_size + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;
            updated_dir.size = Some(dir_size);
            
            // Write the directory
            writer.seek(SeekFrom::Start(next_sector as u64 * SECTOR_SIZE as u64))?;
            updated_dir.write_directory_record(writer)?;
            
            // Update next sector
            next_sector += sectors_needed;
            
            // Store updated directory
            updated_directories.insert(name.clone(), updated_dir);
        }
        
        // Write files
        for (name, file) in &self.files {
            let file_content = file.content();
            let file_size = file_content.len() as u32;
            
            // Write file content
            writer.seek(SeekFrom::Start(next_sector as u64 * SECTOR_SIZE as u64))?;
            writer.write_all(file_content)?;
            
            // Calculate sectors occupied by the file
            let sectors_needed = (file_size + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;
            
            // Create updated file
            let mut updated_file = file.clone();
            updated_file.set_sector_location(next_sector);
            updated_file.set_size(file_size);
            
            // Update next sector
            next_sector += sectors_needed;
            
            // Store updated file
            updated_files.insert(name.clone(), updated_file);
        }
        
        // Now write this directory record
        if let Some(location) = self.sector_location {
            writer.seek(SeekFrom::Start(location as u64 * SECTOR_SIZE as u64))?;
            
            // Create directory record with . and .. entries
            let mut buffer = Vec::new();
            
            // "." entry (current directory)
            let dot_entry = DirectoryEntry::new_directory(".", location, self.calculate_size());
            let entry_size = dot_entry.record_size();
            let mut entry_buffer = vec![0u8; entry_size + 8]; // Add extra padding
            dot_entry.write_to_buffer(&mut entry_buffer[0..entry_size])?;
            buffer.extend_from_slice(&entry_buffer[0..entry_size]);
            
            // ".." entry (parent directory - would be set properly in a real implementation)
            let parent_location = location; // In a real implementation, this would be the parent's location
            let dotdot_entry = DirectoryEntry::new_directory("..", parent_location, 0);
            let entry_size = dotdot_entry.record_size();
            let mut entry_buffer = vec![0u8; entry_size + 8]; // Add extra padding
            dotdot_entry.write_to_buffer(&mut entry_buffer[0..entry_size])?;
            buffer.extend_from_slice(&entry_buffer[0..entry_size]);
            
            // Add entries for subdirectories
            for directory in updated_directories.values() {
                let dir_location = directory.sector_location.unwrap();
                let dir_size = directory.size.unwrap();
                
                let dir_entry = DirectoryEntry::new_directory(directory.name(), dir_location, dir_size);
                let entry_size = dir_entry.record_size();
                let mut entry_buffer = vec![0u8; entry_size + 8]; // Add extra padding
                dir_entry.write_to_buffer(&mut entry_buffer[0..entry_size])?;
                buffer.extend_from_slice(&entry_buffer[0..entry_size]);
            }
            
            // Add entries for files
            for file in updated_files.values() {
                let file_location = file.sector_location().unwrap();
                let file_size = file.size().unwrap();
                
                let file_entry = DirectoryEntry::new_file(file.name(), file_location, file_size);
                let entry_size = file_entry.record_size();
                let mut entry_buffer = vec![0u8; entry_size + 8]; // Add extra padding
                file_entry.write_to_buffer(&mut entry_buffer[0..entry_size])?;
                buffer.extend_from_slice(&entry_buffer[0..entry_size]);
            }
            
            // Pad to sector size
            let padding_needed = SECTOR_SIZE - (buffer.len() % SECTOR_SIZE);
            if padding_needed < SECTOR_SIZE {
                buffer.extend(vec![0u8; padding_needed]);
            }
            
            // Write directory record
            writer.write_all(&buffer)?;
        }
        
        Ok(())
    }
    
    /// Calculate the size of this directory record
    fn calculate_size(&self) -> u32 {
        // Size of "." and ".." entries
        let mut size = self.create_dot_entry().record_size() + self.create_dotdot_entry().record_size();
        
        // Size of subdirectory entries
        for dir in self.directories.values() {
            let entry = DirectoryEntry::new_directory(dir.name(), 0, 0);
            size += entry.record_size();
        }
        
        // Size of file entries
        for file in self.files.values() {
            let entry = DirectoryEntry::new_file(file.name(), 0, 0);
            size += entry.record_size();
        }
        
        size as u32
    }
    
    /// Create a "." directory entry
    fn create_dot_entry(&self) -> DirectoryEntry {
        let location = self.sector_location.unwrap_or(0);
        let size = self.size.unwrap_or(0);
        DirectoryEntry::new_directory(".", location, size)
    }
    
    /// Create a ".." directory entry
    fn create_dotdot_entry(&self) -> DirectoryEntry {
        // In a real implementation, this would use the parent's location
        let location = self.sector_location.unwrap_or(0);
        DirectoryEntry::new_directory("..", location, 0)
    }
    
    /// Write the directory record to a writer
    fn write_directory_record<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Create directory record with . and .. entries
        let mut buffer = Vec::new();
        
        // "." entry (current directory)
        let dot_entry = self.create_dot_entry();
        let mut entry_buffer = vec![0u8; dot_entry.record_size()];
        dot_entry.write_to_buffer(&mut entry_buffer)?;
        buffer.extend_from_slice(&entry_buffer);
        
        // ".." entry (parent directory)
        let dotdot_entry = self.create_dotdot_entry();
        let mut entry_buffer = vec![0u8; dotdot_entry.record_size()];
        dotdot_entry.write_to_buffer(&mut entry_buffer)?;
        buffer.extend_from_slice(&entry_buffer);
        
        // Add entries for subdirectories
        for dir in self.directories.values() {
            if let (Some(location), Some(size)) = (dir.sector_location, dir.size) {
                let dir_entry = DirectoryEntry::new_directory(dir.name(), location, size);
                let mut entry_buffer = vec![0u8; dir_entry.record_size()];
                dir_entry.write_to_buffer(&mut entry_buffer)?;
                buffer.extend_from_slice(&entry_buffer);
            }
        }
        
        // Add entries for files
        for file in self.files.values() {
            if let (Some(location), Some(size)) = (file.sector_location(), file.size()) {
                let file_entry = DirectoryEntry::new_file(file.name(), location, size);
                let mut entry_buffer = vec![0u8; file_entry.record_size()];
                file_entry.write_to_buffer(&mut entry_buffer)?;
                buffer.extend_from_slice(&entry_buffer);
            }
        }
        
        // Pad to sector size
        let padding_needed = SECTOR_SIZE - (buffer.len() % SECTOR_SIZE);
        if padding_needed < SECTOR_SIZE {
            buffer.extend(vec![0u8; padding_needed]);
        }
        
        // Write directory record
        writer.write_all(&buffer)?;
        
        Ok(())
    }
    
    /// Get the directory name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set the directory name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// Get a reference to the subdirectories
    pub fn directories(&self) -> &HashMap<String, Directory> {
        &self.directories
    }
    
    /// Get a mutable reference to the subdirectories
    pub fn directories_mut(&mut self) -> &mut HashMap<String, Directory> {
        &mut self.directories
    }
    
    /// Get a reference to the files
    pub fn files(&self) -> &HashMap<String, File> {
        &self.files
    }
    
    /// Get a mutable reference to the files
    pub fn files_mut(&mut self) -> &mut HashMap<String, File> {
        &mut self.files
    }
    
    /// Find a subdirectory by name
    pub fn find_directory(&self, name: &str) -> Option<&Directory> {
        self.directories.get(name)
    }
    
    /// Find a subdirectory by name and get a mutable reference
    pub fn find_directory_mut(&mut self, name: &str) -> Option<&mut Directory> {
        self.directories.get_mut(name)
    }
    
    /// Find a file by name
    pub fn find_file(&self, name: &str) -> Option<&File> {
        self.files.get(name)
    }
    
    /// Find a file by name and get a mutable reference
    pub fn find_file_mut(&mut self, name: &str) -> Option<&mut File> {
        self.files.get_mut(name)
    }
    
    /// Add a subdirectory
    pub fn add_directory(&mut self, name: String) -> Result<()> {
        if self.directories.contains_key(&name) {
            return Err(Error::PathError(format!("Directory already exists: {}", name)));
        }
        
        let directory = Directory::new(&name);
        self.directories.insert(name, directory);
        
        Ok(())
    }
    
    /// Add a file
    pub fn add_file(&mut self, name: String, content: &[u8]) -> Result<()> {
        if self.files.contains_key(&name) {
            return Err(Error::PathError(format!("File already exists: {}", name)));
        }
        
        let file = File::new(&name, content.to_vec());
        self.files.insert(name, file);
        
        Ok(())
    }
    
    /// Remove a subdirectory
    pub fn remove_directory(&mut self, name: &str) -> Option<Directory> {
        self.directories.remove(name)
    }
    
    /// Remove a file
    pub fn remove_file(&mut self, name: &str) -> Option<File> {
        self.files.remove(name)
    }
    
    /// Convert this directory to a directory entry
    pub fn to_entry(&self) -> DirectoryEntry {
        let location = self.sector_location.unwrap_or(0);
        let size = self.size.unwrap_or_else(|| self.calculate_size());
        
        DirectoryEntry::new_directory(&self.name, location, size)
    }
    
    /// Set the sector location
    pub fn set_sector_location(&mut self, location: u32) {
        self.sector_location = Some(location);
    }
    
    /// Get the sector location
    pub fn sector_location(&self) -> Option<u32> {
        self.sector_location
    }
    
    /// Set the size
    pub fn set_size(&mut self, size: u32) {
        self.size = Some(size);
    }
    
    /// Get the size
    pub fn size(&self) -> Option<u32> {
        self.size
    }
}