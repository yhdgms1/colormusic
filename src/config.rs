use config::Config;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub devices: Option<Vec<String>>,
    pub tcp: Option<bool>,
    pub udp: Option<bool>,
    #[serde(rename = "udp-address")]
    pub udp_address: Option<String>,
}

pub fn get_config() -> Settings {
    let settings = Config::builder()
        .add_source(config::File::with_name("./config"))
        .build()
        .expect("Ошибка при чтении файла конфигурации.");

    settings
        .try_deserialize::<Settings>()
        .expect("Ошибка при попытке разобрать файл конфигурации.")
}
