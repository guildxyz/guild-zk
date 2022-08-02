use config::Config;
use dotenv::dotenv;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub app_port: u16,
    pub apikey: String,
    pub url_balancy: String,
    pub apikey_balancy: String,
    pub url_pubkey: String,
    pub apikey_pubkey: String,
    pub private_key: String,
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

#[derive(serde::Deserialize, Debug)]
pub struct TestSettings {
    #[serde(default = "get_default")]
    pub url_balancy: String,
    #[serde(default = "get_default")]
    pub apikey_balancy: String,
    #[serde(default = "get_default")]
    pub url_pubkey: String,
    #[serde(default = "get_default")]
    pub apikey_pubkey: String,
    #[serde(default = "get_default")]
    pub url_rpc: String,
}

fn get_default() -> String{
    String::from("")
}

pub fn get_test_config() -> TestSettings {
    dotenv().expect("Failed to load dotenv.");

    let conf = Config::builder()
        .add_source(config::Environment::with_prefix("test"))
        .build()
        .expect("Failed to load config.");

    let settings: TestSettings = conf
        .try_deserialize()
        .expect("Failed to deserialize config.");
    settings
}
