use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::info;
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

    /// Data retention period for the backup
    #[clap(short, long, arg_enum, default_value_t =  BackupRetention::SettingsOnly)]
    retention: BackupRetention,

    /// Enables trusting of invalid HTTPS certificates, including self-signed certificates.
    #[clap(short, long)]
    trust_all_certificates: bool,

    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let mut client = OmadaClient::new(&args.base_url, args.trust_all_certificates);
    client.login(&args.username, &args.password)?;

    let name = client.download_backup(BackupRetention::SettingsOnly)?;
    info!("Successfully saved Backup to {}", name);

    Ok(())
}
