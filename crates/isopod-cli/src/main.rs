use clap::Parser;

mod cli;

fn display_entries<Storage>(entry: &isopod::DirectoryRef<'_, Storage>, depth: u64)
where
  Storage: isopod::read::IsoRead,
{
  for entry in entry.entries() {
    if let Ok(entry) = entry {
      match &entry {
        isopod::DirectoryEntryRef::File(file) => {
          println!(
            "{:indent$}File: {}",
            "",
            file.name(),
            indent = (depth * 2) as usize
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
}

fn main() {
  let cli = cli::Cli::parse();

  let file = std::fs::OpenOptions::new()
    .read(true)
    .open(&cli.input)
    .expect("Failed to open input file");

  let iso = isopod::Iso::open(file, isopod::Extensions::all()).expect("Failed to open ISO image");

  if let Some(pv) = iso.primary_volume() {
    display_entries(&pv.root(), 0);
  }
}
