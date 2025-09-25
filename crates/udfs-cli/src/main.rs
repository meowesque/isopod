use clap::Parser;

mod cli;

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
      todo!()
    }
    cli::Command::Info { input } => {
      todo!()
    }
    cli::Command::Validate { input } => {
      todo!()
    }
  }
}
