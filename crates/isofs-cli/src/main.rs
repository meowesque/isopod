use clap::Parser;

mod cli;

fn main() {
  use isofs::writer::*;

  let mut iso = IsoWriter::new(WriterOptions {
    sector_size: 2048,
    standard: Standard::Iso9660,
  });

  let mut file = std::fs::File::create("./data/test-iso9660.iso").unwrap();

  iso.add_volume(Volume::Primary(PrimaryVolume {
    volume_id: "TEST_ISO9660".to_string(),
    publisher: Some("Publisher".to_string()),
    preparer: None,
    filesystem: Filesystem::default(),
  }));

  iso.write(&mut file).unwrap();

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
