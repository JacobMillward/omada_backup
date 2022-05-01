use std::{io::BufReader, path::PathBuf, sync::Arc};

use clap::ArgEnum;
use log::{debug, error};
use normpath::PathExt;
use ureq::Agent;
use url::Url;

use crate::client::helpers::parse_file_name_from_content_disposition;

use super::api_models::*;

#[derive(Clone, ArgEnum)]
pub enum BackupRetention {
    SettingsOnly,
    Days7,
    Days30,
    Days60,
    Days90,
    Days180,
}

pub struct OmadaClient {
    client: Agent,
    base_url: Url,
    csrf_token: Option<String>,
    controller_id: Option<String>,
}

// Helpful definition of Error
type Error = Box<dyn std::error::Error + Sync + Send>;

impl OmadaClient {
    pub fn new(base_url: &str, trust_invalid_certs: bool) -> OmadaClient {
        let tls_connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(trust_invalid_certs)
            .build()
            .unwrap();

        let client = ureq::AgentBuilder::new()
            .tls_connector(Arc::new(tls_connector))
            .build();

        OmadaClient {
            client,
            base_url: Url::parse(base_url).unwrap(),
            csrf_token: None,
            controller_id: None,
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        let api_info: ApiResult<ApiInfo> = self
            .client
            .get(self.base_url.join("api/info")?.as_str())
            .call()?
            .into_json()?;

        if api_info.error_code > 0 {
            error!(
                "Error retrieving API Info: Error {} {}",
                api_info.error_code, api_info.msg
            );
            debug!("{:#?}", api_info);
            return Err(Error::from("Could not retrieve controller ID"));
        }

        let id = api_info.result.unwrap().controller_id;
        debug!("Retrieved Controller ID of {}", &id);
        self.controller_id = Some(id);

        debug!(
            "Logging in to Omada Controller at {}",
            self.base_url.as_str()
        );

        let login: ApiResult<LoginResult> = self
            .client
            .post(
                self.construct_controller_url("api/v2/login")
                    .unwrap()
                    .as_str(),
            )
            .set("Content-Type", "application/json")
            .send_string(
                format!(
                    "{{\"username\":\"{}\",\"password\":\"{}\"}}",
                    username, password
                )
                .as_str(),
            )?
            .into_json()?;

        if login.error_code > 0 {
            error!(
                "Error logging in: Error {} {}",
                api_info.error_code, api_info.msg
            );
            debug!("{:#?}", login);
            return Err(Error::from("Could not login"));
        }

        debug!("{:#?}", login);

        self.csrf_token = Some(login.result.unwrap().token);

        Ok(())
    }

    pub fn download_backup(&self, retention: BackupRetention) -> Result<String, Error> {
        debug!("Preparing Backup");
        let prepare_url_opt =
            self.construct_controller_url("api/v2/maintenance/backup/prepareBackup");
        let backup_url_opt = self.construct_controller_url("api/v2/files/backup");

        if let (Some(prepare_url), Some(mut backup_url), Some(token)) =
            (prepare_url_opt, backup_url_opt, &self.csrf_token)
        {
            let prepare_backup_response: ApiResult<()> = self
                .client
                .post(prepare_url.as_str())
                .set("Csrf-Token", token)
                .call()?
                .into_json()?;

            if prepare_backup_response.error_code > 0 {
                return Err(Error::from(format!(
                    "Could not prepare backup: {}",
                    prepare_backup_response.msg
                )));
            }

            let retention_value = match retention {
                BackupRetention::SettingsOnly => "-1",
                BackupRetention::Days7 => "7",
                BackupRetention::Days30 => "30",
                BackupRetention::Days60 => "60",
                BackupRetention::Days90 => "90",
                BackupRetention::Days180 => "180",
            };

            backup_url
                .query_pairs_mut()
                .append_pair("retention", retention_value);

            let response = self
                .client
                .get(backup_url.as_str())
                .set("Csrf-Token", token)
                .call()?;

            let content_disposition = response.header("Content-Disposition");
            let file_name = parse_file_name_from_content_disposition(content_disposition)
                .unwrap_or_else(|| "Omada_Backup.cfg".to_owned());

            let file_path = PathBuf::from(&file_name);
            let mut backup_file = std::fs::File::create(&file_path)?;

            let normalised_path = file_path
                .normalize()?
                .into_os_string()
                .into_string()
                .unwrap();

            debug!("Writing backup to {:?}", &normalised_path);
            let mut buffered_reader = BufReader::new(response.into_reader());
            std::io::copy(&mut buffered_reader, &mut backup_file)?;

            return Ok(normalised_path);
        }

        Err(Error::from("Not Logged In"))
    }

    fn construct_controller_url(&self, relative_path: &str) -> Option<Url> {
        if let Some(controller_id) = &self.controller_id {
            let url_result = self
                .base_url
                .join(format!("{}/{}", controller_id.as_str(), relative_path).as_str());
            if let Ok(url) = url_result {
                return Some(url);
            }
        }

        None
    }
}
