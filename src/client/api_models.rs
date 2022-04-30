use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResult<T> {
    #[serde(rename = "errorCode")]
    pub error_code: i32,
    pub msg: String,
    pub result: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiInfo {
    #[serde(rename = "controllerVer")]
    pub controller_ver: String,
    #[serde(rename = "apiVer")]
    pub api_ver: String,
    pub configured: bool,
    #[serde(rename = "type")]
    pub _type: i32,
    #[serde(rename = "supportApp")]
    pub support_app: bool,
    #[serde(rename = "omadacId")]
    pub controller_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResult {
    #[serde(rename = "roleType")]
    pub role_type: i32,
    pub token: String,
}
