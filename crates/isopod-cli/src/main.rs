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
      // Handle extract command
    }
    cli::Command::List { input, verbose } => {
      // Handle list command
    }
    cli::Command::Info { input } => {
      // Handle info command
    }
    cli::Command::Validate { input } => {
      // Handle validate command
    }
  }
}
