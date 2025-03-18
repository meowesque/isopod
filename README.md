# isopod

A Rust library and CLI tool for creating, reading, and manipulating ISO 9660 filesystem images.

## Features

- Create ISO 9660 compliant filesystem images
- Read and extract files from existing ISO images
- Support for ISO 9660 extensions:
  - Joliet (long file names, Unicode characters)
  - Rock Ridge (POSIX filesystem features)
  - El Torito (bootable CDs/DVDs)
  - UDF bridge
- Command-line interface for common operations

## Installation

### From crates.io

```bash
cargo install isopod-cli
```

### From source

```bash
git clone https://github.com/meowesque/isopod.git
cd isopod
cargo build --release
```

## Library Usage

Add isopod to your Cargo.toml:

```toml
[dependencies]
isopod = "0.1.0"
```

### Creating an ISO

```rust
use isopod::{Iso, IsoBuilder};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new ISO
    let mut iso = Iso::builder()
        .volume_id("MY_VOLUME")
        .publisher("My Organization")
        .joliet(true)  // Enable Joliet extension for long filenames
        .build()?;
    
    // Add a directory
    iso.add_directory("docs")?;
    
    // Add a file
    iso.add_file("docs/readme.txt", Path::new("path/to/readme.txt"))?;
    
    // Add a file with content
    let content = "Hello, world!".as_bytes();
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an existing ISO
    let iso = Iso::open("input.iso")?;
    
    // Get volume information
    println!("Volume ID: {}", iso.volume_descriptor().volume_id());
    
    // Extract a file
    iso.extract_file("docs/readme.txt", Path::new("extracted_readme.txt"))?;
    
    // Get file content
    let content = iso.get_file_content("hello.txt")?;
    println!("Content: {}", String::from_utf8_lossy(&content));
    
    Ok(())
}
```

## CLI Usage

### Creating an ISO

```bash
isopod create --output my_image.iso --volume-id MY_VOLUME --joliet path/to/files
```

### Extracting files

```bash
# Extract all files
isopod extract --input my_image.iso --output extracted/

# Extract specific files
isopod extract --input my_image.iso --output extracted/ file1.txt docs/file2.txt
```

### Listing contents

```bash
isopod list my_image.iso

# With detailed information
isopod list --verbose my_image.iso
```

### Showing ISO information

```bash
isopod info my_image.iso
```

### Checking ISO validity

```bash
isopod check my_image.iso
```

## ISO 9660 Standard

This library implements the ISO 9660 standard, also known as ECMA-119, which defines the file system for CD-ROMs and other optical media. The implementation focuses on:

- Level 1 compliance (8.3 filenames)
- Level 2 support (32 character filenames)
- Joliet extension for long filenames and Unicode support
- Rock Ridge extension for POSIX filesystem attributes
- El Torito extension for bootable media
- UDF bridge for compatibility with modern systems

## License

This project is licensed under the MIT License - see the LICENSE file for details.