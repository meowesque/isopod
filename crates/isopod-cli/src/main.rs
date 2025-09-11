use clap::Parser;

mod cli;

fn main() {
  let cli = cli::Cli::parse();

  let file = std::fs::read(&cli.input).expect("Failed to read input file");

  let iso = isopod::Iso::new(isopod::Options::default(), file);
 // let volumes = iso.scan_volumes();
  dbg!(iso.read());
}
