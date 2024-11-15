pub enum Mode {
    Colormusic,
    Static,
}

pub struct AppConfig {
    // Или режим цветомузыки или статичный цвет
    pub mode: Mode,
    // Иногда хочется менее яркие цвета
    //
    // Пример использования комманды: `op0.2`
    pub opacity: f32,
    // Пример использования комманды: `sc70.5`
    pub scale: f32
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { mode: Mode::Colormusic, opacity: 1.0, scale: 1.0 }
    }
}