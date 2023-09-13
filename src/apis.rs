use lazy_static::lazy_static;

static BASE_URL: &'static str = "https://api.amazonalexa.com";

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub mod skill_package_management {
    use serde::{Deserialize, Serialize};
    use strum::{Display, EnumString};

    #[derive(Debug, EnumString, Display)]
    #[strum(serialize_all = "camelCase")]
    pub enum SkillStage {
        Development,
        Live,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ExportSkillPackageResponse {
        pub location: String,
        pub export_id: String,
    }

    pub async fn export_skill_package(
        profile_name: &str,
        skill_id: &str,
        stage: SkillStage,
    ) -> Result<ExportSkillPackageResponse, reqwest::Error> {
        let profile = crate::config::CONFIG
            .get_profile(profile_name)
            .expect(format!("Profile '{}' not found in config", profile_name).as_str());

        if !profile.is_valid() {
            panic!("Profile '{}' is not valid:\n{:?}", profile_name, profile);
        }

        let url = format!(
            "{}/v1/skills/{}/stages/{}/exports",
            super::BASE_URL,
            skill_id,
            stage,
        );

        let res = super::HTTP_CLIENT
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", profile.access_token.clone().unwrap()),
            )
            .send()
            .await?;

        let body = res.json::<ExportSkillPackageResponse>().await?;

        Ok(body)
    }
}

pub mod errors {
    use thiserror::Error;
}
