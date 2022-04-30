use std::error::Error;

use omada_backup::client::{OmadaClient, BackupRetention};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut client = OmadaClient::new(OMADA_URL);
    client.login(USERNAME, PASSWORD).await?;
    
    let name = client.download_backup(BackupRetention::SettingsOnly).await?;
    println!("Successfully saved Backup to {:?}", name);

    Ok(())
}
