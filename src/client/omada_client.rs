use std::path::PathBuf;

use clap::ArgEnum;
use log::{debug, error};
use normpath::PathExt;
use reqwest::{
    header::{self, HeaderValue},
    Client, Url,
};

use super::api_models::*;

pub struct OmadaClient {
    client: Client,
    base_url: Url,
    csrf_token: Option<String>,
    controller_id: Option<String>,
}

#[derive(Clone, ArgEnum)]
pub enum BackupRetention {
    SettingsOnly,
    Days7,
    Days30,
    Days60,
    Days90,
    Days180,
}

// Helpful definition of Error
type Error = Box<dyn std::error::Error + Sync + Send>;

impl OmadaClient {
    pub fn new(base_url: &str) -> OmadaClient {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .cookie_store(true)
            .build()
            .unwrap();

        OmadaClient {
            client,
            base_url: Url::parse(base_url).unwrap(),
            csrf_token: None,
            controller_id: None,
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        let api_info = self
            .client
            .get(self.base_url.join("api/info").unwrap())
            .send()
            .await?
            .json::<ApiResult<ApiInfo>>()
            .await?;

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

        let login_response = self
            .client
            .post(self.construct_controller_url("api/v2/login").unwrap())
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        let login = login_response.json::<ApiResult<LoginResult>>().await?;

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

    pub async fn download_backup(&self, retention: BackupRetention) -> Result<String, Error> {
        debug!("Preparing Backup");
        let prepare_url_opt =
            self.construct_controller_url("api/v2/maintenance/backup/prepareBackup");
        let backup_url_opt = self.construct_controller_url("api/v2/files/backup");

        if let (Some(prepare_url), Some(mut backup_url), Some(token)) =
            (prepare_url_opt, backup_url_opt, &self.csrf_token)
        {
            let prepare_backup_response = self
                .client
                .post(prepare_url)
                .header("Csrf-Token", token)
                .send()
                .await?
                .json::<ApiResult<()>>()
                .await?;

            if prepare_backup_response.error_code > 0 {
                return Err(Error::from(String::from(format!(
                    "Could not prepare backup: {}",
                    prepare_backup_response.msg
                ))));
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
                .get(backup_url)
                .header("Csrf-Token", token)
                .send()
                .await?;

            let content_header = response.headers().get(header::CONTENT_DISPOSITION);
            let file_name = parse_file_name_from_content_disposition(content_header)
                .unwrap_or("Omada_Backup.cfg".to_owned());

            let file_path = PathBuf::from(&file_name);
            let mut backup_file = std::fs::File::create(&file_path)?;
            let mut content = std::io::Cursor::new(response.bytes().await?);

            let normalised_path = file_path.normalize()?.into_os_string().into_string().unwrap();
            debug!("Writing backup to {:?}", &normalised_path);
            std::io::copy(&mut content, &mut backup_file)?;

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

fn parse_file_name_from_content_disposition(header: Option<&HeaderValue>) -> Option<String> {
    if let Some(value) = header {
        if value.is_empty() {
            return None;
        }

        let value_str = percent_encoding::percent_decode_str(value.to_str().unwrap())
            .decode_utf8()
            .unwrap();
        let index = value_str.find("fileName=").map(|i| i + 9);

        if let Some(start) = index {
            return Some(value_str[start..].to_owned());
        }
    }

    None
}
