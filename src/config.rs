use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub bridge_address: String,
    pub app_key: String,
    pub on_button_rid: String,
    pub off_button_rid: String,
    pub light_id: String,
}

pub fn get_config() -> Result<AppConfig, config::ConfigError> {
    let config = Config::builder()
        .add_source(config::Environment::with_prefix("HUE").try_parsing(true))
        .build()?;

    config.try_deserialize::<AppConfig>()
}
