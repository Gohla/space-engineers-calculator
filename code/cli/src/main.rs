use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use steamlocate::SteamDir;

use secalc_core::data::Data;
use secalc_core::data::extract::ExtractConfig;

#[derive(Parser, Debug)]
#[command(name = "SECalc", about = "Space Engineers Calculator")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Extracts game data into a format that SECalc can handle
  ExtractGameData {
    #[arg(long, short, env = "SECALC_EXTRACT_SE_DIRECTORY")]
    /// Space Engineers directory to extract game data from. Automatically inferred if installed via Steam when not set
    se_directory: Option<PathBuf>,
    #[arg(long, env = "SECALC_EXTRACT_SE_WORKSHOP_DIRECTORY")]
    /// Space engineers workshop (mod) directory. Automatically inferred if installed via Steam when not set. No mods are extracted if this directory is not found
    se_workshop_directory: Option<PathBuf>,
    #[arg(env = "SECALC_EXTRACT_CONFIG_FILE")]
    /// Extract configuration file
    config_file: PathBuf,
    /// File to write extracted data to
    #[arg(env = "SECALC_EXTRACT_OUTPUT_FILE")]
    output_file: PathBuf,
  },
}

fn main() -> Result<()> {
  dotenv::dotenv()
    .context("Failed to read .env file")?;
  let cli = Cli::parse();
  match cli.command {
    Command::ExtractGameData {
      se_directory,
      se_workshop_directory,
      config_file,
      output_file
    } => {
      let mut steam_dir = SteamDir::locate();
      let se_directory = se_directory.or(get_se_directory(&mut steam_dir))
        .context("Space Engineers directory was not set, and failed to automatically infer the directory")?;
      let se_workshop_directory = se_workshop_directory.or(get_se_workshop_directory(&se_directory));
      let config_reader = File::open(config_file)
        .context("Failed to open extract config file for reading")?;
      let extract_config: ExtractConfig = ron::de::from_reader(config_reader)
        .context("Failed to read extract configuration")?;
      let data = Data::extract_from_se_dir(se_directory, se_workshop_directory, extract_config)
        .context("Failed to read Space Engineers data")?;
      let data_writer = OpenOptions::new().write(true).create(true).truncate(true).open(output_file)
        .context("Failed to create a writer for writing game data to file")?;
      data.to_json(data_writer)
        .context("Failed to write game data to file")?;
    }
  }
  Ok(())
}

fn get_se_directory(steam_dir: &mut Option<SteamDir>) -> Option<PathBuf> {
  steam_dir.as_mut().and_then(|steam_dir| steam_dir.app(&244850).map(|app| app.path.clone()))
}

fn get_se_workshop_directory(se_directory: &PathBuf) -> Option<PathBuf> {
  se_directory.parent().and_then(|common_dir| common_dir.parent().map(|steamapps_dir| steamapps_dir.join("workshop/content/244850")))
}
