use lt_rs::settings_pack::SettingsPack;

pub struct AnawtOptions {
    /// How often to update torrent status
    pub(crate) tick_rate: u64,
    pub(crate) settings_pack: Option<SettingsPack>,
}

impl AnawtOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick_rate(mut self, tick_rate: u64) -> Self {
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
            tick_rate: 2,
            settings_pack: None,
        }
    }
}
