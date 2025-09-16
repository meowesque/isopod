use clap::Parser;

mod cli;

fn main() {
  let cli = cli::Cli::parse();

  let file = std::fs::OpenOptions::new()
    .read(true)
    .open(&cli.input)
    .expect("Failed to open input file");

  let iso = isopod::Iso::open(file).expect("Failed to open ISO image");

  for volume in iso.volumes() {
    match volume {
      isopod::VolumeRef::Primary(volume) => {
        println!("Primary Volume: {}", volume.identifier().as_ref());

        for entry in volume.root().entries() {
          match entry {
            Ok(isopod::DirectoryEntryRef::Directory(d)) => {
              println!("Directory: {}", d.name().as_ref());
            }
            Ok(isopod::DirectoryEntryRef::File(f)) => {
              println!("File: {} (ver. {})", f.name(), f.revision());
            }
            Err(e) => {
              eprintln!("{}", e);
              break;
            }
          }
        }
      }
      isopod::VolumeRef::Supplementary(_) => {}
    }
  }
}
