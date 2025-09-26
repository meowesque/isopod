use clap::Parser;

mod cli;

fn main() {
  use isofs::writer::*;

  let mut iso = IsoWriter::new(WriterOptions {
    sector_size: 2048,
    standard: Standard::Iso9660,
  });

  let mut filesystem = isofs::writer::fs::Filesystem::default();

  filesystem.upsert_file("a/b/c", "./data/file1.txt").unwrap();

  filesystem
    .upsert_file("a/b/c.txt", "./data/file1.txt")
    .unwrap();

  dbg!(&filesystem);

  iso.add_volume(isofs::writer::volume::PrimaryVolume {
    volume_id: "TEST_ISO9660".to_string(),
    publisher: Some("Publisher".to_string()),
    preparer: None,
    filesystem,
  });

  let file = std::fs::File::create("./data/test-iso9660.iso").unwrap();
  let mut writer = std::io::BufWriter::new(file);

  iso
    .write(&mut writer)
    .unwrap();

  /*let cli = cli::Cli::parse();

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
      todo!()
    }
    cli::Command::Info { input } => {
      todo!()
    }
    cli::Command::Validate { input } => {
      todo!()
    }
  }*/
}
