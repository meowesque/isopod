use std::{path::Path, rc::Rc};

use arraystring::{
  typenum::{U128, U32},
  ArrayString,
};

use crate::spec;

#[derive(Debug)]
struct LbaAllocator {
  sector_size: u16,
  next_lba: u32,
  // allocations: HashMap<u32, u32>,
}

impl LbaAllocator {
  fn new(sector_size: u16, start_lba: u32) -> Self {
    Self {
      sector_size,
      next_lba: start_lba,
      // allocations: HashMap::new(),
    }
  }

  fn allocate(&mut self, size: usize) -> u32 {
    let size_in_sectors = size.div_ceil(self.sector_size as usize) as u32;
    self.allocate_sectors(size_in_sectors)
  }

  fn allocate_sectors(&mut self, size_in_sectors: u32) -> u32 {
    let allocated_lba = self.next_lba;
    // self.allocations.insert(allocated_lba, size_in_sectors);
    self.next_lba += size_in_sectors;
    allocated_lba as u32
  }
}

#[derive(Default, Debug)]
pub struct WriterMetadata {
  publisher_identifier: Option<ArrayString<U32>>,
  data_preparer_identifier: Option<ArrayString<U128>>,
  application_identifier: Option<ArrayString<U128>>,
  copyright_file_identifier: Option<ArrayString<U32>>,
  abstract_file_identifier: Option<[u8; 37]>,
}

impl WriterMetadata {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn publisher_identifier(mut self, identifier: impl Into<ArrayString<U32>>) -> Self {
    self.publisher_identifier = Some(identifier.into());
    self
  }

  pub fn data_preparer_identifier(mut self, identifier: impl Into<ArrayString<U128>>) -> Self {
    self.data_preparer_identifier = Some(identifier.into());
    self
  }

  pub fn application_identifier(mut self, identifier: impl Into<ArrayString<U128>>) -> Self {
    self.application_identifier = Some(identifier.into());
    self
  }

  pub fn copyright_file_identifier(mut self, identifier: impl Into<ArrayString<U32>>) -> Self {
    self.copyright_file_identifier = Some(identifier.into());
    self
  }

  pub fn abstract_file_identifier(mut self, identifier: [u8; 37]) -> Self {
    self.abstract_file_identifier = Some(identifier);
    self
  }
}

#[derive(Debug)]
pub struct WriterOptions {
  pub sector_size: u16,
  pub metadata: WriterMetadata,
}

impl Default for WriterOptions {
  fn default() -> Self {
    Self {
      sector_size: 2048,
      metadata: WriterMetadata::new(),
    }
  }
}

/// Reference to file content.
///
/// This is likely to change in the future, given the various ways to provide file content.
#[derive(Debug, Clone)]
pub enum FileContentRef {
  Handle(Rc<std::fs::File>),
  Memory(Vec<u8>),
}

#[derive(Debug, Clone)]
struct FsFile {
  lba: Option<u32>,
  /// File name (max 128 bytes for Joliet)
  name: ArrayString<U128>,
  content: FileContentRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FsDirectoryName {
  Root,
  Current,
  Parent,
  /// Directory name (max 128 bytes for Joliet)
  Name(ArrayString<U128>),
}

#[derive(Debug, Clone)]
struct FsDirectory {
  lba: Option<u32>,
  name: FsDirectoryName,
  children: Vec<FsNode>,
}

impl FsDirectory {
  /// Insert a node into this directory.
  ///
  /// If a node with the same name already exists, it will
  /// be merged (for directories) or replaced (for files).
  pub fn insert(&mut self, node: &FsNode) {
    // TODO(meowesque): Potentially using a better data structure here
    // TODO(meowesque): would make this a little more optimal.
    // TODO(meowesque): Speaking of optimal, see if we can avoid cloning and simply move.

    for child in &mut self.children {
      match (child, &node) {
        (FsNode::Directory(child_dir), FsNode::Directory(node_dir))
          // TODO(meowesque): How should we handle comparison of `..` and `.`?
          if child_dir.name == node_dir.name =>
        {
          // If a directory with the same name exists, merge their children.
          node_dir.children.iter().for_each(|x| child_dir.insert(x));
          return;
        }
        (FsNode::File(child_file), FsNode::File(file)) if child_file.name == file.name => {
          // If a file with the same name exists, just replace it.
          let _ = std::mem::replace(child_file, file.clone());
          return;
        }
        _ => {}
      }
    }

    self.children.push(node.clone());
  }
}

#[derive(Debug, Clone)]
enum FsNode {
  File(FsFile),
  Directory(FsDirectory),
}

impl FsNode {
  pub fn name(&self) -> &str {
    match self {
      FsNode::File(file) => &file.name,
      FsNode::Directory(dir) => match &dir.name {
        FsDirectoryName::Root => "/",
        FsDirectoryName::Current => ".",
        FsDirectoryName::Parent => "..",
        FsDirectoryName::Name(name) => &name,
      },
    }
  }

  fn calculate_record_size(&self) -> usize {
    let size_template = |name_length: usize| {
      1 + // Length of Directory Record
      1 + // Extended Attribute Record Length
      8 + // Location of Extent (LBA)
      8 + // Data Length
      7 + // Recording Date and Time
      1 + // File Flags
      1 + // File Unit Size
      1 + // Interleave Gap Size
      4 + // Volume Sequence Number
      1 + // Length of File Identifier
      if name_length > 0 { name_length } else { 1 } + // File Identifier
      1 + // Padding field
      0 // System Use (not used)
    };

    let name_length = match self {
      FsNode::File(file) => file.name.len() as usize,
      FsNode::Directory(dir) => match &dir.name {
        // TODO(meowesque): How long is Root? 0?
        FsDirectoryName::Root => 1,
        FsDirectoryName::Current => 1,
        FsDirectoryName::Parent => 1,
        FsDirectoryName::Name(name) => name.len() as usize,
      },
    };

    // TODO(meowesque): Handle this error possibility better
    // TODO(meowesque): Use ArrayString<N> where N is is `255 - size_template(0) - 1``
    assert!(255 >= size_template(0) + name_length - 1);

    size_template(name_length)
  }

  /// Calculate the total extent (in bytes) required to store this node's content.
  fn calculate_extent(&self) -> usize {
    match self {
      FsNode::File(file) => match &file.content {
        FileContentRef::Handle(file) => file.metadata().map(|m| m.len() as usize).unwrap_or(0),
        FileContentRef::Memory(vec) => vec.len(),
      },
      FsNode::Directory(dir) => dir.children.iter().map(Self::calculate_record_size).sum(),
    }
  }

  /// Recursively allocate LBAs for this node and its children.
  fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    let extent = self.calculate_extent();
    let lba = allocator.allocate(extent);

    match self {
      FsNode::File(file) => file.lba = Some(lba),
      FsNode::Directory(dir) => {
        dir.lba = Some(lba);

        for child in &mut dir.children {
          child.allocate_lbas(allocator);
        }
      }
    }
  }

  fn lba(&self) -> Option<u32> {
    match self {
      FsNode::File(file) => file.lba,
      FsNode::Directory(dir) => dir.lba,
    }
  }

  fn into_directory_record(&self) -> spec::DirectoryRecord {
    spec::DirectoryRecord {
      record_length: self.calculate_record_size() as u8,
      extended_attribute_record_length: 0,
      extent_lba: self
        .lba()
        .expect("DirectoryRecord must have an allocated LBA"),
      extent_length: self.calculate_extent() as u32,
      recording_date: spec::IsoDateTime {
        years_since_1900: 123, // TODO(meowesque): Current year - 1900
        month: 1,              // TODO(meowesque): Current month
        day: 1,                // TODO(meowesque): Current day
        hour: 0,               // TODO(meowesque): Current hour
        minute: 0,             // TODO(meowesque): Current minute
        second: 0,             // TODO(meowesque): Current second
        offset: 0, // TODO(meowesque): Current timezone offset in 15-minute intervals from GMT
      },
      file_flags: match self {
        FsNode::File(_) => spec::FileFlags::empty(),
        FsNode::Directory(_) => spec::FileFlags::DIRECTORY,
      },
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 1,
      identifier_length: match self {
        FsNode::File(file) => file.name.len() as u8,
        FsNode::Directory(dir) => match &dir.name {
          FsDirectoryName::Root => 1,
          FsDirectoryName::Current => 1,
          FsDirectoryName::Parent => 1,
          FsDirectoryName::Name(name) => name.len() as u8,
        },
      },
      // TODO(meowesque): Sanitize names to be valid ISO 9660 identifiers.
      identifier: match self {
        FsNode::File(file) => self.name().to_owned(),
        FsNode::Directory(dir) => match &dir.name {
          FsDirectoryName::Root => "\u{0}".to_owned(),
          FsDirectoryName::Current => "\u{0}".to_owned(),
          FsDirectoryName::Parent => "\u{1}".to_owned(),
          FsDirectoryName::Name(name) => name.as_str().to_owned(),
        },
      },
    }
  }
}

#[derive(Debug)]
struct Filesystem {
  root: FsDirectory,
}

impl Filesystem {
  pub fn new() -> Self {
    Self {
      root: FsDirectory {
        lba: None,
        name: FsDirectoryName::Root,
        children: vec![],
      },
    }
  }

  pub fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.root.lba = Some(allocator.allocate(34)); // TODO(meowesque): Unreadable?

    for child in &mut self.root.children {
      child.allocate_lbas(allocator);
    }
  }
}

#[derive(Debug)]
pub struct IsoWriter {
  options: WriterOptions,
  filesystem: Filesystem,
}

impl IsoWriter {
  pub fn new(options: WriterOptions) -> Self {
    Self {
      options,
      filesystem: Filesystem::new(),
    }
  }

  pub fn options(&self) -> &WriterOptions {
    &self.options
  }

  /// Insert a file into the layout at the specified path with the given size.
  ///
  /// If intermediate directories do not exist, they will be created.
  pub fn insert_file(&mut self, path: impl AsRef<Path>, content: FileContentRef) {
    // TODO(meowesque): Create an IsoPath type that enforces valid UTF-8 and max length?
    // TODO(meowesque): Also enforce max length of 128 bytes for Joliet.

    let path = path.as_ref();
    let components: Vec<_> = path.components().collect();
    let name = components
      .last()
      .expect("Path must have at least one component")
      .as_os_str()
      .to_str()
      .expect("File name must be valid UTF-8")
      .to_owned();

    let mut tail = FsNode::File(FsFile {
      lba: None,
      // TODO(meowesque): Handle error properly
      name: ArrayString::try_from(name.as_str()).expect("File name too long"),
      content,
    });

    for component in path.components().rev().skip(1) {
      tail = FsNode::Directory(FsDirectory {
        lba: None,
        // TODO(meowesque): Handle error properly, parse special names, etc.
        name: FsDirectoryName::Name(
          ArrayString::try_from(
            component
              .as_os_str()
              .to_str()
              .expect("Directory name must be valid UTF-8"),
          )
          .expect("Directory name too long"),
        ),
        children: vec![tail],
      });
    }

    // TODO(meowesque): Implement this functionality *within* Filesystem, instead of IsoWriter.
    self.filesystem.root.insert(&tail);
  }

  pub fn finalize<Out>(mut self, out: Out) -> Result<(), crate::error::Error>
  where
    Out: std::io::Write + std::io::Seek,
  {
    let mut allocator = LbaAllocator::new(
      self.options.sector_size,
      /* TODO(meowesque): Arbitrary? */ 20,
    );

    self.filesystem.allocate_lbas(&mut allocator);

    let writer = std::io::BufWriter::new(out);
    let mut _cursor = std::io::Cursor::new(writer);

    let pvd = spec::PrimaryVolumeDescriptor {
        standard_identifier: todo!(),
        version: todo!(),
        system_identifier: todo!(),
        volume_identifier: todo!(),
        volume_space_size: todo!(),
        volume_set_size: todo!(),
        volume_sequence_number: todo!(),
        logical_block_size: todo!(),
        path_table_size: todo!(),
        type_l_path_table_lba: todo!(),
        optional_type_l_path_table_lba: todo!(),
        type_m_path_table_lba: todo!(),
        optional_type_m_path_table_lba: todo!(),
        root_directory_record: todo!(),
        volume_set_identifier: todo!(),
        publisher_identifier: todo!(),
        data_preparer_identifier: todo!(),
        application_identifier: todo!(),
        copyright_file_identifier: todo!(),
        abstract_file_identifier: todo!(),
        bibliographic_file_identifier: todo!(),
        volume_creation_date: todo!(),
        volume_modification_date: todo!(),
        volume_expiration_date: todo!(),
        volume_effective_date: todo!(),
        file_structure_version: todo!(),
        application_data: todo!(),
        reserved: todo!(),
    };

    // TODO(meowesque): Implement

    Ok(())
  }
}
