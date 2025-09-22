use clap::Parser;

mod cli;

/*
fn display_entries<Storage>(entry: &isopod::DirectoryRef<'_, Storage>, depth: u64)
where
  Storage: isopod::read::IsoRead,
{
  for entry in entry.entries() {
    if let Ok(entry) = entry {
      match &entry {
        isopod::DirectoryEntryRef::File(file) => {
          println!(
            "{:indent$}File: {} {}",
            "",
            file.name(),
            file.size(),
            indent = (depth * 2) as usize,
          );
        }
        isopod::DirectoryEntryRef::Directory(dir) => {
          println!(
            "{:indent$}Directory: {}",
            "",
            dir.name(),
            indent = (depth * 2) as usize
          );
          display_entries(dir, depth + 1);
        }
      }
    }
  }
} */

fn list(input: std::path::PathBuf, _verbose: bool) {
  let file = std::fs::File::open(&input).expect("Failed to open input ISO file");
  let iso =
    isopod::Iso::open(file, isopod::Extensions::all()).expect("Failed to read ISO filesystem");

  if let Some(bvd) = iso.boot_record_volume() {
    println!("Boot Volume Descriptor: {:?}", bvd.descriptor());
  }
}

fn main() {
  let cli = cli::Cli::parse();

  match cli.command {
    cli::Command::Create {
      output,
      volume_id,
      publisher,
      preparer,
      files,
      joliet,
      rock_ridge,
    } => {
      todo!()
    }
    cli::Command::Extract { input, output } => {
      todo!()
    }
    cli::Command::List { input, verbose } => {
      list(input, verbose);
    }
    cli::Command::Info { input } => {
      todo!()
    }
    cli::Command::Validate { input } => {
      todo!()
    }
  }
}
