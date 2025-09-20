use serde::Deserialize;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

mod config_dir;
pub use config_dir::{find_config_file, read_config};

mod error;
pub use error::{ConfigError, ConfigResult};
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize)]
pub struct Config {
    host: Host,
    app: App,
}

#[derive(Debug, Deserialize)]
pub struct Host {
    bindto: String,
}

#[derive(Debug, Deserialize)]
pub struct App {
    jwt: String,
    database_uri: String,
}

impl Config {
    #[tracing::instrument]
    pub async fn get_or_init(use_local: bool) -> &'static Config {
        CONFIG
            .get_or_init(|| async {
                let read_cfg = |use_local| -> ConfigResult<Self> {
                    let bytes = read_config(use_local)?;
                    let config: Self = toml::from_slice(&bytes)?;
                    Ok(config)
                };

                let config = match read_cfg(use_local) {
                    Ok(c) => c,
                    Err(e) => {
                        if !matches!(e, error::ConfigError::ConfigNotFound) {
                            crate::error::log_error(&e);
                        }
                        tracing::error!("Config not found.");
                        std::process::exit(1);
                    }
                };

                config
            })
            .await
    }

    #[inline]
    pub fn host(&self) -> &Host {
        &self.host
    }

    #[inline]
    pub fn app(&self) -> &App {
        &self.app
    }
}

impl Host {
    #[inline]
    pub fn bindto(&self) -> &str {
        &self.bindto
    }
}

impl App {
    #[inline]
    pub fn jwt(&self) -> &str {
        &self.jwt
    }

    #[inline]
    pub fn database_uri(&self) -> &str {
        &self.database_uri
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn config_test() {
        let config = Config::get_or_init(true).await;
        assert_eq!(config.host().bindto(), "127.0.0.1:5000"); // defaults
    }
}
