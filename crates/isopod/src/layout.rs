use std::{collections::HashMap, path::PathBuf, rc::Rc};

use arraystring::ArrayString;

use crate::spec;

#[derive(Debug, Clone)]
pub enum FileContentRef {
  // TODO(meowesque): Use PathBuf/File here instead of just size.
  FsFile(usize),
  // TODO(meowesque): Allow in-memory content?
  // Pod(Rc<Vec<u8>>)
}

#[derive(Debug)]
pub struct LbaAllocator {
  sector_size: u16,
  next_lba: u32,
  allocations: HashMap<u32, u32>,
}

impl LbaAllocator {
  pub fn new(sector_size: u16, start_lba: u32) -> Self {
    Self {
      sector_size,
      next_lba: start_lba,
      allocations: HashMap::new(),
    }
  }

  pub fn allocate(&mut self, size: usize) -> u32 {
    let size_in_sectors = size.div_ceil(self.sector_size as usize) as u32;
    self.allocate_sectors(size_in_sectors)
  }

  pub fn allocate_sectors(&mut self, size_in_sectors: u32) -> u32 {
    let allocated_lba = self.next_lba;
    self.allocations.insert(allocated_lba, size_in_sectors);
    self.next_lba += size_in_sectors;
    allocated_lba as u32
  }
}

#[derive(Debug, Clone)]
pub struct FsFile {
  lba: Option<u32>,
  name: String,
  content: FileContentRef,
}

#[derive(Debug, Clone)]
pub struct FsDirectory {
  lba: Option<u32>,
  name: String,
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
pub enum FsNode {
  File(FsFile),
  Directory(FsDirectory),
}

impl FsNode {
  pub fn name(&self) -> &str {
    match self {
      FsNode::File(file) => &file.name,
      FsNode::Directory(dir) => &dir.name,
    }
  }

  pub(crate) fn calculate_record_size(&self) -> usize {
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
      FsNode::File(file) => file.name.len(),
      FsNode::Directory(dir) => dir.name.len(),
    };

    // TODO(meowesque): Handle this error possibility better
    // TODO(meowesque): Use ArrayString<N> where N is is `255 - size_template(0) - 1``
    assert!(255 >= size_template(0) + name_length - 1);

    size_template(name_length)
  }

  /// Calculate the total extent (in bytes) required to store this node's content.
  pub(crate) fn calculate_extent(&self) -> usize {
    match self {
      FsNode::File(file) => match &file.content {
        FileContentRef::FsFile(size) => *size,
      },
      FsNode::Directory(dir) => dir.children.iter().map(Self::calculate_record_size).sum(),
    }
  }

  pub fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
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
}

pub struct PrimaryVolumeInfo {}

#[derive(Debug)]
pub struct IsoLayout {
  sector_size: u16,
  pub root: FsNode,
}

impl IsoLayout {
  pub fn new(sector_size: u16) -> Self {
    Self {
      sector_size,
      root: FsNode::Directory(FsDirectory {
        lba: None,
        name: "".to_owned(),
        children: vec![],
      }),
    }
  }

  /// Insert a file into the layout at the specified path with the given size.
  ///
  /// If intermediate directories do not exist, they will be created.
  pub fn insert_file(&mut self, path: &PathBuf, size: usize) {
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
      name,
      content: FileContentRef::FsFile(size),
    });

    for component in path.components().rev().skip(1) {
      tail = FsNode::Directory(FsDirectory {
        lba: None,
        name: component
          .as_os_str()
          .to_str()
          .expect("Directory name must be valid UTF-8")
          .to_owned(),
        children: vec![tail],
      });
    }

    let FsNode::Directory(dir) = &mut self.root else {
      unreachable!("Root node is a directory");
    };

    dir.insert(&tail);
  }
}
