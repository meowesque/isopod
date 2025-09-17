use clap::Parser;

mod cli;

fn main() {
  let cli = cli::Cli::parse();

  let file = std::fs::OpenOptions::new()
    .read(true)
    .open(&cli.input)
    .expect("Failed to open input file");

  let iso = isopod::Iso::open(file, isopod::Extensions::all()).expect("Failed to open ISO image");

  match iso.primary_volume() {
    Some(pvd) => {
      println!("{:?}", pvd.descriptor());
    }
    None => {
      println!("No Primary Volume Descriptor found.");
    }
  }

  match iso.supplementary_volume() {
    Some(svd) => {
      println!("{:?}", svd.descriptor());
    }
    None => {
      println!("No Supplementary Volume Descriptor found.");
    }
  }
}
