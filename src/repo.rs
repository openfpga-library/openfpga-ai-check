use base64::prelude::*;
use octocrab::{Octocrab, repos::RepoHandler};

pub struct RepoContext {
    pub client: Octocrab,
    pub owner: String,
    pub repo: String,
}

impl RepoContext {
    pub fn repo(&self) -> anyhow::Result<RepoHandler<'_>> {
        Ok(self.client.repos(&self.owner, &self.repo))
    }

    pub async fn readme_text(&self) -> anyhow::Result<String> {
        let readme_result = self.repo()?.get_readme().send().await?;

        match (
            readme_result.content,
            readme_result
                .encoding
                .as_ref()
                .and_then(|s| Some(s.as_str())),
        ) {
            (Some(encoded_text), Some("base64")) => {
                let clean_b64 = encoded_text.replace('\n', "");

                Ok(String::from_utf8(
                    BASE64_STANDARD.decode(clean_b64.as_bytes())?,
                )?)
            }
            _ => Ok("".to_string()),
        }
    }
}
