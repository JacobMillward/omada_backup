use clap::Parser;
use clap_verbosity_flag::{Verbosity, InfoLevel};
use log::info;
use std::error::Error;

use omada_backup::client::{OmadaClient, BackupRetention};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// User to login to the Omada Controller as
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

    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Args::parse();

    env_logger::Builder::new()
    .filter_level(args.verbose.log_level_filter())
    .init();

    let mut client = OmadaClient::new(&args.base_url);
    client.login(&args.username, &args.password).await?;
    
    let name = client.download_backup(BackupRetention::SettingsOnly).await?;
    info!("Successfully saved Backup to {}", name);

    Ok(())
}