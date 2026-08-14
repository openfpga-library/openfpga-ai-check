use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoresResponse {
    pub data: Vec<Core>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Core {
    pub id: String,
    pub repository: Repository,
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub platform: String,
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub download_url: String,
    pub requires_license: bool,
    pub updaters: Option<UpdatersFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatersFile {
    pub declare_ai: Option<Vec<String>>,
}

pub async fn request_library_info() -> Result<Vec<Core>, Error> {
    let url = "https://openfpga-library.github.io/analogue-pocket/api/v3/cores.json";
    let response: CoresResponse = reqwest::get(url).await?.json().await?;
    Ok(response.data)
}
