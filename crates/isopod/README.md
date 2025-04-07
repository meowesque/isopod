# isopod

A Rust library for creating, reading, and manipulating ISO 9660 filesystem images.

## Overview

isopod implements the ISO 9660 standard (ECMA-119) for optical disc filesystems. It allows you to:

- Create new ISO 9660 images
- Read and extract files from existing ISO images
- Modify ISO image content
- Support multiple ISO 9660 extensions

## Usage Examples

### Creating a New ISO

```rust
use isopod::{Iso, IsoBuilder};
use std::path::Path;

fn main() -> isopod::Result<()> {
    // Create a new ISO with custom properties
    let mut iso = Iso::builder()
        .volume_id("MY_VOLUME")
        .publisher("Publisher Name")
        .preparer("Data Preparer")
        .application("My Application")
        .joliet(true)  // Enable Joliet extension
        .build()?;
    
    // Add directories
    iso.add_directory("docs")?;
    iso.add_directory("docs/images")?;
    
    // Add files from the filesystem
    iso.add_file("docs/readme.txt", Path::new("local/path/to/readme.txt"))?;
    iso.add_file("docs/images/logo.png", Path::new("local/path/to/logo.png"))?;
    
    // Add a file with content
    let content = b"Hello, world!";
    iso.add_file_with_content("hello.txt", content)?;
    
    // Save the ISO
    iso.save("output.iso")?;
    
    Ok(())
}
```

### Reading an ISO

```rust
use isopod::Iso;
use std::path::Path;

fn main() -> isopod::Result<()> {
    // Open an existing ISO
    let iso = Iso::open("input.iso")?;
    
    // Get volume information
    println!("Volume ID: {}", iso.volume_descriptor().volume_id());
    println!("Publisher: {}", iso.volume_descriptor().publisher_id());
    
    // Extract a file
    iso.extract_file("docs/readme.txt", Path::new("extracted_readme.txt"))?;
    
    // Get file content
    let content = iso.get_file_content("hello.txt")?;
    println!("Content: {}", String::from_utf8_lossy(&content));
    
    // Print file listing
    print_directory(iso.root_directory(), Path::new(""));
    
    Ok(())
}

fn print_directory(dir: &isopod::Directory, path: &Path) {
    // Print files
    for (name, _) in dir.files() {
        println!("File: {}", path.join(name).display());
    }
    
    // Print and traverse subdirectories
    for (name, subdir) in dir.directories() {
        println!("Directory: {}/", path.join(name).display());
        print_directory(subdir, &path.join(name));
    }
}
```

## Features

### ISO 9660 Extensions

isopod supports several extensions to the basic ISO 9660 standard:

#### Joliet

Adds support for:
- Long file names (up to 64 characters)
- Unicode characters
- Deeper directory hierarchies

Enable with:

```rust
Iso::builder().joliet(true).build()?;
```

#### Rock Ridge

Adds POSIX filesystem features:
- File permissions
- Symbolic links
- Deep directory hierarchies
- Long file names

Enable with:

```rust
Iso::builder().rock_ridge(true).build()?;
```

#### El Torito

Adds support for bootable CDs/DVDs.

Enable with:

```rust
Iso::builder().el_torito(true).build()?;
```

#### UDF Bridge

Universal Disk Format support for compatibility with modern systems.

Enable with:

```rust
Iso::builder().udf(true).build()?;
```

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

## Standards Compliance

This library aims to implement the ISO 9660 standard (ECMA-119) correctly, with additional support for common extensions.

### Limitations

- Maximum file size: 4GB (32-bit size limit in ISO 9660)
- Default filename restrictions follow ISO 9660 Level 1:
  - Up to 8 characters for filename + 3 for extension
  - Uppercase letters, numbers, and underscore only
  - These restrictions are lifted when using extensions like Joliet or Rock Ridge

## Performance Considerations

- Creating ISOs with many small files can be slow due to the need to write each file's content to a separate sector
- For best performance when creating large ISOs, add files in sorted order