use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Result, Context, bail};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, error, warn, debug};
use walkdir::WalkDir;

use isopod::{Iso, IsoBuilder};

/// Command-line interface for isopod ISO library
#[derive(Parser)]
#[clap(
    name = "isopod",
    about = "Create and manipulate ISO 9660 filesystem images",
    version,
    author
)]
struct Cli {
    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,
    
    /// Subcommand to execute
    #[clap(subcommand)]
    command: Command,
}

/// Available commands
#[derive(Subcommand)]
enum Command {
    /// Create a new ISO image
    Create {
        /// Output ISO file
        #[clap(short, long)]
        output: PathBuf,
        
        /// Volume ID
        #[clap(short = 'i', long, default_value = "ISO_VOLUME")]
        volume_id: String,
        
        /// Publisher ID
        #[clap(short, long)]
        publisher: Option<String>,
        
        /// Data preparer
        #[clap(short = 'r', long)]
        preparer: Option<String>,
        
        /// Input directory or files to include
        #[clap(required = true)]
        input: Vec<PathBuf>,
        
        /// Enable Joliet extension for long filenames
        #[clap(long)]
        joliet: bool,
        
        /// Enable Rock Ridge extension for POSIX filesystem features
        #[clap(long)]
        rock_ridge: bool,
        
        /// Enable El Torito extension for bootable CDs
        #[clap(long)]
        el_torito: bool,
        
        /// Enable UDF bridge
        #[clap(long)]
        udf: bool,
    },
    
    /// Extract files from an ISO image
    Extract {
        /// Input ISO file
        #[clap(short, long)]
        input: PathBuf,
        
        /// Output directory
        #[clap(short, long, default_value = ".")]
        output: PathBuf,
        
        /// Specific files to extract
        files: Vec<String>,
    },
    
    /// List contents of an ISO image
    List {
        /// Input ISO file
        input: PathBuf,
        
        /// Show detailed information
        #[clap(short, long)]
        verbose: bool,
    },
    
    /// Show information about an ISO image
    Info {
        /// Input ISO file
        input: PathBuf,
    },
    
    /// Check the validity of an ISO image
    Check {
        /// Input ISO file
        input: PathBuf,
    },
}

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();
    
    // Initialize logger
    init_logger(cli.verbose);
    
    // Execute command
    if let Err(err) = run_command(cli.command) {
        error!("Error: {}", err);
        
        // Add more context to the error if possible
        if let Some(source) = err.source() {
            error!("Caused by: {}", source);
        }
        
        process::exit(1);
    }
}

/// Initialize the logger
fn init_logger(verbose: bool) {
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", if verbose { "debug" } else { "info" });
    
    let _ = env_logger::try_init_from_env(env);
}

/// Run the selected command
fn run_command(command: Command) -> Result<()> {
    match command {
        Command::Create { 
            output, 
            volume_id, 
            publisher, 
            preparer, 
            input, 
            joliet, 
            rock_ridge, 
            el_torito, 
            udf 
        } => {
            create_iso(output, volume_id, publisher, preparer, input, joliet, rock_ridge, el_torito, udf)
        },
        Command::Extract { input, output, files } => {
            extract_from_iso(input, output, files)
        },
        Command::List { input, verbose } => {
            list_iso_contents(input, verbose)
        },
        Command::Info { input } => {
            show_iso_info(input)
        },
        Command::Check { input } => {
            check_iso(input)
        },
    }
}

/// Create a new ISO image
fn create_iso(
    output: PathBuf,
    volume_id: String,
    publisher: Option<String>,
    preparer: Option<String>,
    input: Vec<PathBuf>,
    joliet: bool,
    rock_ridge: bool,
    el_torito: bool,
    udf: bool,
) -> Result<()> {
    info!("Creating ISO image: {}", output.display());
    
    // Validate inputs
    if input.is_empty() {
        bail!("No input files or directories specified");
    }
    
    // Check if we can write to the output path
    if let Some(parent) = output.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }
    }
    
    // Build ISO
    info!("Setting up ISO builder");
    let mut builder = Iso::builder()
        .volume_id(volume_id)
        .joliet(joliet)
        .rock_ridge(rock_ridge)
        .el_torito(el_torito)
        .udf(udf);
    
    if let Some(publisher) = publisher {
        builder = builder.publisher(publisher);
    }
    
    if let Some(preparer) = preparer {
        builder = builder.preparer(preparer);
    }
    
    info!("Creating ISO structure");
    let mut iso = builder.build()
        .with_context(|| "Failed to create ISO structure")?;
    
    // Add files and directories
    let progress = create_progress_bar("Adding files");
    
    let mut found_files = false;
    for path in input {
        debug!("Processing input path: {}", path.display());
        
        // Handle wildcard paths by expanding them
        if path.to_string_lossy().contains('*') {
            debug!("Wildcard path detected: {}", path.display());
            
            // Get the parent directory of the wildcard
            let parent = match path.parent() {
                Some(p) if p.as_os_str().is_empty() => Path::new("."),
                Some(p) => p,
                None => Path::new("."),
            };
            
            // Get the filename pattern
            let pattern = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };
            
            debug!("Wildcard parent: {}, pattern: {}", parent.display(), pattern);
            
            // List all files in parent and match against pattern
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    let name = match entry_path.file_name() {
                        Some(name) => name.to_string_lossy().to_string(),
                        None => continue,
                    };
                    
                    // Simple wildcard matching
                    if matches_wildcard(&name, &pattern) {
                        debug!("Wildcard match: {}", entry_path.display());
                        if let Err(e) = add_files_to_iso(&mut iso, &entry_path, &entry_path, &progress) {
                            warn!("Error adding {}: {}", entry_path.display(), e);
                        } else {
                            found_files = true;
                        }
                    }
                }
            }
        } else {
            // Regular path
            if let Err(e) = add_files_to_iso(&mut iso, &path, &path, &progress) {
                warn!("Error adding {}: {}", path.display(), e);
            } else {
                found_files = true;
            }
        }
    }
    
    if !found_files {
        bail!("No valid files found to add to the ISO");
    }
    
    progress.finish_with_message("Files added");
    
    // Save ISO
    info!("Writing ISO image to disk");
    let save_progress = create_progress_bar("Writing ISO");
    save_progress.set_message(format!("Writing to {}", output.display()));
    
    match iso.save(&output) {
        Ok(_) => {
            save_progress.finish_with_message(format!("ISO created: {}", output.display()));
            Ok(())
        },
        Err(e) => {
            save_progress.finish_with_message("Failed to write ISO");
            Err(e).with_context(|| format!("Failed to write ISO to {}", output.display()))
        }
    }
}

/// Simple wildcard pattern matching
fn matches_wildcard(name: &str, pattern: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('*').collect();
    
    // If the pattern doesn't contain wildcard, just compare directly
    if pattern_parts.len() == 1 {
        return name == pattern;
    }
    
    // Check if name starts with the first part (if not empty)
    if !pattern_parts[0].is_empty() && !name.starts_with(pattern_parts[0]) {
        return false;
    }
    
    // Check if name ends with the last part (if not empty)
    if !pattern_parts.last().unwrap().is_empty() && !name.ends_with(pattern_parts.last().unwrap()) {
        return false;
    }
    
    // For middle parts, check if they appear in order
    let mut remaining = name;
    for &part in &pattern_parts[..pattern_parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        
        match remaining.find(part) {
            Some(pos) => {
                remaining = &remaining[pos + part.len()..];
            },
            None => return false,
        }
    }
    
    true
}


/// Add files to the ISO
fn add_files_to_iso(
    iso: &mut Iso,
    base_path: &Path,
    current_path: &Path,
    progress: &ProgressBar,
) -> Result<()> {
    // Check if the path exists
    if !current_path.exists() {
        warn!("Path does not exist: {}", current_path.display());
        return Ok(());
    }
    
    if current_path.is_dir() {
        // Create the directory in the ISO (if not root)
        if current_path != base_path {
            let rel_path = match current_path.strip_prefix(base_path) {
                Ok(path) => path,
                Err(_) => {
                    // This can happen with wildcards or special characters
                    // In this case, just use the filename part
                    match current_path.file_name() {
                        Some(name) => Path::new(name),
                        None => {
                            warn!("Could not determine relative path for: {}", current_path.display());
                            return Ok(());
                        }
                    }
                }
            };
            
            progress.set_message(format!("Adding directory {}", rel_path.display()));
            iso.add_directory(rel_path)?;
        }
        
        // Add all files and subdirectories
        match std::fs::read_dir(current_path) {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let path = entry.path();
                            if let Err(e) = add_files_to_iso(iso, base_path, &path, progress) {
                                warn!("Error adding {}: {}", path.display(), e);
                            }
                        },
                        Err(e) => {
                            warn!("Error reading directory entry: {}", e);
                        }
                    }
                }
            },
            Err(e) => {
                warn!("Error reading directory {}: {}", current_path.display(), e);
            }
        }
    } else if current_path.is_file() {
        // Add file to ISO
        let rel_path = match current_path.strip_prefix(base_path) {
            Ok(path) => path,
            Err(_) => {
                // This can happen with wildcards or special characters
                // In this case, just use the filename part
                match current_path.file_name() {
                    Some(name) => Path::new(name),
                    None => {
                        warn!("Could not determine relative path for: {}", current_path.display());
                        return Ok(());
                    }
                }
            }
        };
        
        progress.set_message(format!("Adding file {}", rel_path.display()));
        progress.inc(1);
        
        match iso.add_file(rel_path, current_path) {
            Ok(_) => {},
            Err(e) => {
                warn!("Error adding file {}: {}", current_path.display(), e);
            }
        }
    }
    
    Ok(())
}

/// Extract files from an ISO
fn extract_from_iso(
    input: PathBuf,
    output: PathBuf,
    files: Vec<String>,
) -> Result<()> {
    info!("Extracting from ISO: {}", input.display());
    
    // Create output directory if it doesn't exist
    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }
    
    // Open ISO
    let iso = Iso::open(&input)?;
    
    // Extract files
    let progress = create_progress_bar("Extracting files");
    
    if files.is_empty() {
        // Extract all files
        extract_directory(
            iso.root_directory(), 
            &output, 
            &PathBuf::new(),
            &progress
        )?;
    } else {
        // Extract specific files
        for file_path in files {
            let path = PathBuf::from(file_path);
            let target_path = output.join(&path);
            
            progress.set_message(format!("Extracting {}", path.display()));
            progress.inc(1);
            
            // Create parent directory if needed
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Extract file
            iso.extract_file(&path, &target_path)
                .with_context(|| format!("Failed to extract {}", path.display()))?;
        }
    }
    
    progress.finish_with_message("Files extracted");
    
    Ok(())
}

/// Extract a directory recursively
fn extract_directory(
    directory: &isopod::Directory,
    output_base: &Path,
    rel_path: &Path,
    progress: &ProgressBar,
) -> Result<()> {
    // Create current directory
    let current_dir = output_base.join(rel_path);
    std::fs::create_dir_all(&current_dir)?;
    
    // Extract files
    for (name, file) in directory.files() {
        let file_rel_path = rel_path.join(name);
        let file_path = output_base.join(&file_rel_path);
        
        progress.set_message(format!("Extracting {}", file_rel_path.display()));
        progress.inc(1);
        
        // Write file content
        std::fs::write(&file_path, file.content())?;
    }
    
    // Extract subdirectories
    for (name, dir) in directory.directories() {
        let dir_rel_path = rel_path.join(name);
        extract_directory(dir, output_base, &dir_rel_path, progress)?;
    }
    
    Ok(())
}

/// List contents of an ISO
fn list_iso_contents(
    input: PathBuf,
    verbose: bool,
) -> Result<()> {
    info!("Listing contents of ISO: {}", input.display());
    
    // Open ISO
    let iso = Iso::open(&input)?;
    
    // Print volume info
    println!("Volume ID: {}", iso.volume_descriptor().volume_id());
    println!("Publisher: {}", iso.volume_descriptor().publisher_id());
    println!("Preparer: {}", iso.volume_descriptor().preparer_id());
    println!("Application: {}", iso.volume_descriptor().application_id());
    println!();
    
    // Print file listing
    println!("Contents:");
    list_directory(iso.root_directory(), &PathBuf::new(), verbose, 0)?;
    
    Ok(())
}

/// List a directory recursively
fn list_directory(
    directory: &isopod::Directory,
    path: &Path,
    verbose: bool,
    indent: usize,
) -> Result<()> {
    // List files
    for (name, file) in directory.files() {
        let file_path = path.join(name);
        
        if verbose {
            println!(
                "{:indent$}{} [{} bytes]",
                "", 
                file_path.display(), 
                file.content().len(),
                indent = indent
            );
        } else {
            println!("{:indent$}{}", "", file_path.display(), indent = indent);
        }
    }
    
    // List subdirectories
    for (name, dir) in directory.directories() {
        let dir_path = path.join(name);
        
        println!("{:indent$}{}/ [directory]", "", dir_path.display(), indent = indent);
        
        // Recursively list subdirectory
        list_directory(dir, &dir_path, verbose, indent + 2)?;
    }
    
    Ok(())
}

/// Show information about an ISO
fn show_iso_info(input: PathBuf) -> Result<()> {
    info!("Showing information for ISO: {}", input.display());
    
    // Open ISO
    let iso = Iso::open(&input)?;
    
    // Print volume info
    println!("ISO 9660 Image Information");
    println!("=========================");
    println!("File: {}", input.display());
    println!("Volume ID: {}", iso.volume_descriptor().volume_id());
    println!("Publisher: {}", iso.volume_descriptor().publisher_id());
    println!("Data Preparer: {}", iso.volume_descriptor().preparer_id());
    println!("Application: {}", iso.volume_descriptor().application_id());
    
    // Print file count
    let (file_count, dir_count, total_size) = count_items(iso.root_directory());
    println!("File Count: {}", file_count);
    println!("Directory Count: {}", dir_count);
    println!("Total Data Size: {} bytes", total_size);
    
    // Print extensions
    let extensions = iso.extensions();
    println!("\nExtensions:");
    println!("  Joliet: {}", if extensions.joliet { "Yes" } else { "No" });
    println!("  Rock Ridge: {}", if extensions.rock_ridge { "Yes" } else { "No" });
    println!("  El Torito: {}", if extensions.el_torito { "Yes" } else { "No" });
    println!("  UDF: {}", if extensions.udf { "Yes" } else { "No" });
    
    Ok(())
}

/// Count items in a directory
fn count_items(directory: &isopod::Directory) -> (usize, usize, usize) {
    let mut file_count = directory.files().len();
    let mut dir_count = directory.directories().len();
    let mut total_size = directory.files().values().map(|f| f.content().len()).sum();
    
    // Recursively count subdirectories
    for subdir in directory.directories().values() {
        let (files, dirs, size) = count_items(subdir);
        file_count += files;
        dir_count += dirs;
        total_size += size;
    }
    
    (file_count, dir_count, total_size)
}

/// Check the validity of an ISO
fn check_iso(input: PathBuf) -> Result<()> {
    info!("Checking ISO: {}", input.display());
    
    // First check basic file access
    let file_size = match std::fs::metadata(&input) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            bail!("Cannot access ISO file: {}", e);
        }
    };
    
    if file_size == 0 {
        bail!("ISO file is empty");
    }
    
    info!("ISO file size: {} bytes", file_size);
    
    // Try to open and parse the ISO
    let iso = match Iso::open(&input) {
        Ok(iso) => {
            info!("Successfully parsed ISO basic structure");
            iso
        },
        Err(err) => {
            bail!("Invalid ISO format: {}", err);
        }
    };
    
    // Basic validation
    println!("ISO Check Summary");
    println!("================");
    println!("File: {}", input.display());
    println!("Size: {} bytes", file_size);
    
    let volume_id = iso.volume_descriptor().volume_id();
    println!("Volume ID: {}", volume_id);
    if volume_id.is_empty() {
        warn!("ISO has empty volume ID");
    }
    
    // Check for standard directories
    let root = iso.root_directory();
    
    // Count and validate files
    let progress = create_progress_bar("Checking files");
    progress.set_message("Scanning directory structure");
    
    let result = validate_directory(root, &PathBuf::new(), &progress);
    progress.finish_with_message("ISO check complete");
    
    // Print summary
    if let Ok((file_count, dir_count, errors)) = result {
        println!("Files found: {}", file_count);
        println!("Directories found: {}", dir_count);
        
        if errors.is_empty() {
            println!("Status: Valid (no errors found)");
        } else {
            println!("Status: Invalid ({} errors found)", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
    
    Ok(())
}

/// Validate a directory recursively
fn validate_directory(
    directory: &isopod::Directory,
    path: &Path,
    progress: &ProgressBar,
) -> Result<(usize, usize, Vec<String>)> {
    let mut file_count = 0;
    let mut dir_count = 0;
    let mut errors = Vec::new();
    
    // Check files
    for (name, file) in directory.files() {
        let file_path = path.join(name);
        progress.set_message(format!("Checking {}", file_path.display()));
        progress.inc(1);
        
        // Basic file validation
        if name.is_empty() {
            errors.push(format!("File at {} has empty name", file_path.display()));
        }
        
        file_count += 1;
    }
    
    // Check subdirectories
    for (name, dir) in directory.directories() {
        let dir_path = path.join(name);
        
        // Basic directory validation
        if name.is_empty() {
            errors.push(format!("Directory at {} has empty name", dir_path.display()));
        }
        
        // Recursively validate subdirectory
        let (files, dirs, subdir_errors) = validate_directory(dir, &dir_path, progress)?;
        file_count += files;
        dir_count += dirs + 1;
        errors.extend(subdir_errors);
    }
    
    Ok((file_count, dir_count, errors))
}

/// Create a progress bar with the given message
fn create_progress_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    pb
}