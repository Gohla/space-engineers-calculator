use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use steamlocate::SteamDir;
use structopt::StructOpt;

use secalc_core::data::Data;
use secalc_core::data::extract::ExtractConfig;

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
    /// Space Engineers directory to extract game data from. Automatically inferred if installed via Steam when not set
    se_directory: Option<PathBuf>,
    #[structopt(long, parse(from_os_str))]
    /// Space engineers workshop (mod) directory. Automatically inferred if installed via Steam when not set. No mods are extracted if this directory is not found
    se_workshop_directory: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    /// Extract configuration file
    extract_config_file: PathBuf,
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
      se_workshop_directory,
      extract_config_file,
      output_file
    } => {
      let mut steam_dir = SteamDir::locate();
      let se_directory = se_directory.or(get_se_directory(&mut steam_dir)).expect("Space Engineers directory was not set, and failed to automatically infer the directory");
      let se_workshop_directory = se_workshop_directory.or(get_se_workshop_directory(&se_directory));
      let extract_config: ExtractConfig = ron::de::from_reader(File::open(extract_config_file).unwrap()).expect("Failed to read extract configuration");
      let data = Data::extract_from_se_dir(se_directory, se_workshop_directory, extract_config).expect("Failed to read Space Engineers data");
      let writer = OpenOptions::new().write(true).create(true).truncate(true).open(output_file).expect("Failed to create a writer for writing game data to file");
      data.to_json(writer).expect("Failed to write game data to file");
    },
  }
}

fn get_se_directory(steam_dir: &mut Option<SteamDir>) -> Option<PathBuf> {
  steam_dir.as_mut().and_then(|steam_dir| steam_dir.app(&244850).map(|app| app.path.clone()))
}

fn get_se_workshop_directory(se_directory: &PathBuf) -> Option<PathBuf> {
  se_directory.parent().and_then(|common_dir| common_dir.parent().map(|steamapps_dir| steamapps_dir.join("workshop/content/244850")))
}
