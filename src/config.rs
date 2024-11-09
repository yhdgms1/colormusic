use config::Config;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub devices: Option<Vec<String>>,
    pub tcp: Option<bool>,
    #[serde(rename = "tcp-port")]
    pub tcp_port: Option<String>,
    pub udp: Option<bool>,
    #[serde(rename = "udp-address")]
    pub udp_address: Option<String>,
    #[serde(rename = "udp-port")]
    pub udp_port: Option<String>,
    #[serde(rename = "color-change-interval")]
    pub color_change_interval: Option<u64>,
}

pub fn get_config() -> Settings {
    let settings = Config::builder()
        .add_source(config::File::with_name("./config.json").format(config::FileFormat::Json))
        .build()
        .expect("Ошибка при чтении файла конфигурации.");

    settings
        .try_deserialize::<Settings>()
        .expect("Ошибка при попытке разобрать файл конфигурации.")
}
