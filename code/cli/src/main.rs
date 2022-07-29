use std::fs::OpenOptions;
use std::path::PathBuf;

use structopt::StructOpt;

use secalc_core::data::Data;

#[derive(Debug, StructOpt)]
#[structopt(name = "SECalc", about = "Space Engineers Calculator")]
struct Opt {
  #[structopt(subcommand)]
  command: Command
}

#[derive(Debug, StructOpt)]
enum Command {
  #[structopt()]
  /// Extracts game data into a format that SECalc can handle
  ExtractGameData {
    #[structopt(long, short, parse(from_os_str))]
    /// Space Engineers directory to extract game data from. Automatically inferred if not set
    se_directory: Option<PathBuf>,
    /// File to write extracted data to
    #[structopt(parse(from_os_str))]
    output_file: PathBuf,
  },
}

fn main() {
  let opt: Opt = Opt::from_args();
  match opt.command {
    Command::ExtractGameData {
      se_directory,
      output_file
    } => {
      let se_directory = se_directory.or(get_se_path()).expect("Space Engineers directory was not set, and failed to automatically infer the directory");
      let data = Data::extract_from_se_dir(se_directory).expect("Failed to read Space Engineers data");
      let writer = OpenOptions::new().write(true).create(true).truncate(true).open(output_file).expect("Failed to create a writer for writing game data to file");
      data.to_json(writer).expect("Failed to write game data to file");
    },
  }
}

#[cfg(windows)]
fn get_se_path() -> Option<PathBuf> {
  use winreg::enums::*;
  use winreg::RegKey;

  let se_app = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 244850").ok()?;
  let se_path: String = se_app.get_value("InstallLocation").ok()?;
  Some(se_path.into())
}

#[cfg(not(windows))]
fn get_se_path() -> Option<PathBuf> { None }
