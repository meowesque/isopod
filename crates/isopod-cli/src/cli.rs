use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[arg(short, long)]
  pub input: PathBuf,
}
