# isopod-cli

Command-line interface for the isopod ISO 9660 library.

## Installation

### From crates.io

```bash
cargo install isopod-cli
```

### From source

```bash
git clone https://github.com/meowesque/isopod.git
cd isopod
cargo install --path crates/isopod-cli
```

## Usage

### Creating an ISO

Create a new ISO image containing files and directories:

```bash
isopod create --output my_image.iso --volume-id MY_VOLUME path/to/files
```

Options:
- `--output`, `-o`: Output ISO file path (required)
- `--volume-id`, `-i`: Volume identifier (default: "ISO_VOLUME")
- `--publisher`, `-p`: Publisher identifier
- `--preparer`, `-r`: Data preparer identifier
- `--joliet`: Enable Joliet extension for long filenames
- `--rock-ridge`: Enable Rock Ridge extension for POSIX filesystem features
- `--el-torito`: Enable El Torito extension for bootable CDs
- `--udf`: Enable UDF bridge
- `--verbose`, `-v`: Enable verbose output

Example with multiple options:

```bash
isopod create --output my_image.iso \
  --volume-id "MY_VOLUME" \
  --publisher "My Organization" \
  --preparer "isopod" \
  --joliet \
  --verbose \
  path/to/files
```

### Extracting Files

Extract files from an ISO image:

```bash
# Extract all files
isopod extract --input my_image.iso --output extracted/

# Extract specific files
isopod extract --input my_image.iso --output extracted/ file1.txt docs/file2.txt
```

Options:
- `--input`, `-i`: Input ISO file path (required)
- `--output`, `-o`: Output directory (default: current directory)
- `--verbose`, `-v`: Enable verbose output

### Listing Contents

List contents of an ISO image:

```bash
isopod list my_image.iso

# With detailed information
isopod list --verbose my_image.iso
```

Options:
- `--verbose`, `-v`: Show detailed information (file sizes, dates)

### Showing ISO Information

Show detailed information about an ISO image:

```bash
isopod info my_image.iso
```

This command displays:
- Volume ID, publisher, preparer, application
- File and directory counts
- Total data size
- Enabled extensions

### Checking ISO Validity

Check the validity of an ISO image:

```bash
isopod check my_image.iso
```

This command verifies:
- ISO 9660 structure correctness
- Volume descriptor validity
- Path table correctness
- Directory records integrity

## Examples

### Creating a bootable ISO

```bash
isopod create --output bootable.iso \
  --volume-id "BOOT_DISK" \
  --el-torito \
  boot_files/
```

### Creating an ISO with unicode filenames

```bash
isopod create --output unicode.iso \
  --volume-id "UNICODE" \
  --joliet \
  files_with_unicode_names/
```

### Extracting specific files maintaining directory structure

```bash
isopod extract --input archive.iso \
  --output extracted/ \
  docs/manual.pdf \
  images/logo.png
```

## Exit Codes

- `0`: Success
- `1`: Error

## Reporting Issues

If you encounter any problems or have suggestions, please open an issue on the GitHub repository:

https://github.com/maxine-deandrade/isopod/issues