use clap::Parser;
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
    #[clap(short = 'l', long)]
    url: String,

    /// Data retention period for the backup
    #[clap(short, long, arg_enum, default_value_t =  BackupRetention::SettingsOnly)]
    retention: BackupRetention,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Args::parse();

    let mut client = OmadaClient::new(&args.url);
    client.login(&args.username, &args.password).await?;
    
    let name = client.download_backup(BackupRetention::SettingsOnly).await?;
    println!("Successfully saved Backup to {:?}", name);

    Ok(())
}
