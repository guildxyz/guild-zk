use config::Config;
use dotenv::dotenv;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub app_port: u16,
    pub apikey: String,
}

pub fn get_config() -> Settings {
    dotenv().expect("Failed to load dotenv.");

    let conf = Config::builder()
        .add_source(config::Environment::default())
        .build()
        .expect("Failed to load config.");

    let settings: Settings = conf
        .try_deserialize()
        .expect("Failed to deserialize config.");
    settings
}
