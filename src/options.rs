use std::time::Duration;

use lt_rs::settings_pack::SettingsPack;

pub struct AnawtOptions {
    /// How often to update torrent status
    pub tick_rate: Duration,
    pub settings_pack: Option<SettingsPack>,
}

impl AnawtOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// How often to update torrent status
    pub fn tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    pub fn settings_pack(mut self, settings_pack: SettingsPack) -> Self {
        self.settings_pack = Some(settings_pack);
        self
    }
}

impl Default for AnawtOptions {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(500),
            settings_pack: None,
        }
    }
}
