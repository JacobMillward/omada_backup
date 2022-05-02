use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{error, info};
use std::error::Error;

use omada_backup::client::{BackupRetention, OmadaClient};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// User to login to the Omada Controller
    #[clap(short, long)]
    username: String,

    /// Password for the User
    #[clap(short, long)]
    password: String,

    /// Base URL for the Omada SDN Controller
    #[clap(short = 'b', long)]
    base_url: String,

    /// Write to file instead of current directory
    #[clap(short, long)]
    output_file: Option<String>,

    /// Data retention period for the backup
    #[clap(short, long, arg_enum, default_value_t =  BackupRetention::SettingsOnly)]
    retention: BackupRetention,

    /// Enables trusting of invalid HTTPS certificates, including self-signed certificates.
    #[clap(short, long)]
    trust_all_certificates: bool,

    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

fn main() {
    let args = Args::parse();

    std::process::exit(match get_backup(args) {
        Ok(output_name) => {
            info!("Successfully saved backup to {}", output_name);
            0
        }
        Err(error) => {
            error!("{}", error.to_string());
            1
        }
    })
}

fn get_backup(args: Args) -> Result<String, Box<dyn Error + Sync + Send>> {
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let mut client = OmadaClient::new(&args.base_url, args.trust_all_certificates);
    client.login(&args.username, &args.password)?;

    let name = client.download_backup(args.output_file, BackupRetention::SettingsOnly)?;

    Ok(name)
}
