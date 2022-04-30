use std::error::Error;

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

pub enum BackupRetention {
    SettingsOnly,
    Days7,
    Days30,
    Days60,
    Days90,
    Days180,
}

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

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), reqwest::Error> {
        let api_info = self
            .client
            .get(self.base_url.join("api/info").unwrap())
            .send()
            .await?
            .json::<ApiResult<ApiInfo>>()
            .await?
            .result
            .unwrap();

        self.controller_id = Some(api_info.controller_id);

        let login_response = self
            .client
            .post(self.construct_controller_url("api/v2/login").unwrap())
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        let login = login_response
            .json::<ApiResult<LoginResult>>()
            .await?
            .result
            .unwrap();

        self.csrf_token = Some(login.token);

        Ok(())
    }

    pub async fn download_backup(
        &self,
        retention: BackupRetention,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        if let Some(token) = &self.csrf_token {
            let prepare_url = self
                .construct_controller_url("api/v2/maintenance/backup/prepareBackup")
                .unwrap();

            let prepare_backup_response = self
                .client
                .post(prepare_url)
                .header("Csrf-Token", token)
                .send()
                .await?
                .json::<ApiResult<()>>()
                .await?;

            if prepare_backup_response.error_code > 0 {
                return Err(Box::<dyn Error + Sync + Send>::from(String::from(format!(
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

            let mut backup_url = self
                .construct_controller_url("api/v2/files/backup")
                .unwrap();

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

            let mut backup_file = std::fs::File::create(&file_name)?;
            let mut content = std::io::Cursor::new(response.bytes().await?);
            std::io::copy(&mut content, &mut backup_file)?;

            return Ok(file_name);
        }

        Err(Box::<dyn Error + Sync + Send>::from(String::from(
            "Not Logged In",
        )))
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
