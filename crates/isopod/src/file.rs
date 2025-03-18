use std::time::SystemTime;

/// Represents a file in the ISO 9660 filesystem
#[derive(Debug, Clone)]
pub struct File {
    /// File name
    name: String,
    
    /// File content
    content: Vec<u8>,
    
    /// Sector location in the ISO
    sector_location: Option<u32>,
    
    /// File size in bytes
    size: Option<u32>,
    
    /// Creation time
    creation_time: SystemTime,
    
    /// Modification time
    modification_time: SystemTime,
}

impl File {
    /// Create a new file
    pub fn new(name: &str, content: Vec<u8>) -> Self {
        let now = SystemTime::now();
        
        Self {
            name: name.to_string(),
            content,
            sector_location: None,
            size: None,
            creation_time: now,
            modification_time: now,
        }
    }
    
    /// Get the file name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set the file name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// Get the file content
    pub fn content(&self) -> &[u8] {
        &self.content
    }
    
    /// Get a mutable reference to the file content
    pub fn content_mut(&mut self) -> &mut Vec<u8> {
        &mut self.content
    }
    
    /// Set the file content
    pub fn set_content(&mut self, content: Vec<u8>) {
        let size = content.len() as u32;
        self.content = content;
        self.size = Some(size);
    }
    
    /// Get the sector location
    pub fn sector_location(&self) -> Option<u32> {
        self.sector_location
    }
    
    /// Set the sector location
    pub fn set_sector_location(&mut self, location: u32) {
        self.sector_location = Some(location);
    }
    
    /// Get the file size
    pub fn size(&self) -> Option<u32> {
        self.size.or_else(|| Some(self.content.len() as u32))
    }
    
    /// Set the file size
    pub fn set_size(&mut self, size: u32) {
        self.size = Some(size);
    }
    
    /// Get the creation time
    pub fn creation_time(&self) -> SystemTime {
        self.creation_time
    }
    
    /// Set the creation time
    pub fn set_creation_time(&mut self, time: SystemTime) {
        self.creation_time = time;
    }
    
    /// Get the modification time
    pub fn modification_time(&self) -> SystemTime {
        self.modification_time
    }
    
    /// Set the modification time
    pub fn set_modification_time(&mut self, time: SystemTime) {
        self.modification_time = time;
    }
}