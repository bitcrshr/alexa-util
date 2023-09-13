use chrono::{DateTime, Utc};
use home::home_dir;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref CONFIG_PATH: std::path::PathBuf = {
        let homedir = home_dir().expect("Unable to find home directory");
        let config_dir = homedir.join(".alexa-util").join("config.json");
        config_dir
    };
    pub static ref CONFIG: Config = Config::new().expect("Unable to load config");
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord, PartialEq, Hash, Serialize, Deserialize)]
pub struct Config {
    profiles: Vec<ConfigProfile>,
}

impl Config {
    pub fn new() -> Result<Self, errors::ConfigError> {
        match Self::exists()? {
            true => Ok(Self::from_file()?),

            false => {
                let config = Self {
                    profiles: Vec::new(),
                };

                config.write()?;
                Ok(config)
            }
        }
    }

    pub fn get_profile(&self, name: &str) -> Option<&ConfigProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn add_profile(&mut self, profile: &ConfigProfile) -> Result<(), errors::ConfigError> {
        if self.get_profile(&profile.name).is_some() {
            return Err(errors::ConfigError::AlreadyExists);
        }

        self.profiles.push(profile.clone());
        self.write()?;
        Ok(())
    }

    pub fn write(&self) -> Result<(), errors::ConfigError> {
        let config_path = CONFIG_PATH.clone();
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        let file = std::fs::File::create(CONFIG_PATH.clone())?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<(), errors::ConfigError> {
        let config = Config::from_file()?;
        self.profiles = config.profiles.clone();
        Ok(())
    }

    pub fn from_file() -> Result<Self, errors::ConfigError> {
        let file = std::fs::File::open(CONFIG_PATH.clone())?;
        let config: Config = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn exists() -> Result<bool, errors::ConfigError> {
        Ok(CONFIG_PATH.exists())
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        match self.write() {
            Ok(_) => (),
            Err(err) => eprintln!("Failed to write config: {:?}", err),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConfigProfile {
    pub name: String,
    pub vendor_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: String,

    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl ConfigProfile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            vendor_id: None,
            access_token: None,
            refresh_token: None,
            token_type: String::from("Bearer"),
            expires_at: None,
        }
    }

    pub fn init(
        &mut self,
        access_token: String,
        refresh_token: String,
        expires_in: u64,
        vendor_id: String,
    ) {
        self.vendor_id = Some(vendor_id);
        self.access_token = Some(access_token);
        self.refresh_token = Some(refresh_token);
        self.expires_at = Some(Utc::now() + chrono::Duration::seconds(expires_in as i64));
    }

    pub fn is_initialized(&self) -> bool {
        self.access_token.is_some()
    }

    pub fn is_valid(&self) -> bool {
        self.is_initialized()
            && match self.expires_at {
                Some(expires_at) => expires_at > Utc::now(),
                None => false,
            }
    }
}

pub mod errors {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum ConfigError {
        #[error("Config already exists")]
        AlreadyExists,

        #[error("Config not found")]
        NotFound,

        #[error("A filesystem error occurred")]
        IoError(#[from] std::io::Error),

        #[error("Failed to either serialize or deserialize config")]
        SerdeError(#[from] serde_json::Error),
    }
}
