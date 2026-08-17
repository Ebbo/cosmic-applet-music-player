use cosmic_config::{Config, ConfigGet, ConfigSet};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CONFIG_VERSION: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled_players: HashSet<String>,
    pub auto_detect_new_players: bool,
    pub selected_player: Option<String>,
    pub show_all_players: bool,
    pub hide_inactive_players: bool,
    #[serde(default)]
    pub custom_play_icon: Option<String>,
    #[serde(default)]
    pub custom_pause_icon: Option<String>,
    #[serde(default)]
    pub custom_next_icon: Option<String>,
    #[serde(default)]
    pub custom_previous_icon: Option<String>,
    pub show_controls: bool,
    #[serde(default)]
    pub show_info_pane: bool,
    #[serde(default)]
    pub info_pane_left: bool, //true = Left, false = Right
    #[serde(default)]
    pub info_pane_width: u32,
    #[serde(default)]
    pub album_size: u32,


}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled_players: HashSet::new(),
            auto_detect_new_players: true,
            selected_player: None,
            show_all_players: false,
            hide_inactive_players: false,
            custom_play_icon: None,
            custom_pause_icon: None,
            custom_next_icon: None,
            custom_previous_icon: None,
            show_controls: false,
            show_info_pane: false,
            info_pane_left: true,
            info_pane_width: 250,
            album_size: 70
        }
    }
}

pub struct ConfigManager {
    config: Config,
    app_config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::new("com.github.MusicPlayer", CONFIG_VERSION)?;
        let app_config = match config.get::<AppConfig>("config") {
            Ok(config) => config,
            Err(_) => {
                let default_config = AppConfig::default();
                config.set("config", &default_config)?;
                default_config
            }
        };

        Ok(Self { config, app_config })
    }

    pub fn get_selected_player(&self) -> Option<String> {
        self.app_config.selected_player.clone()
    }

    pub fn set_selected_player(&mut self, player: Option<String>) -> anyhow::Result<()> {
        self.app_config.selected_player = player;
        self.save_config()
    }

    pub fn get_auto_detect_new_players(&self) -> bool {
        self.app_config.auto_detect_new_players
    }

    pub fn set_auto_detect_new_players(&mut self, auto_detect: bool) -> anyhow::Result<()> {
        self.app_config.auto_detect_new_players = auto_detect;
        self.save_config()
    }

    pub fn add_discovered_player(&mut self, player_name: String) -> anyhow::Result<()> {
        if self.app_config.auto_detect_new_players {
            self.app_config.enabled_players.insert(player_name);
            self.save_config()?;
        }
        Ok(())
    }

    pub fn get_show_all_players(&self) -> bool {
        self.app_config.show_all_players
    }

    pub fn set_show_all_players(&mut self, show_all: bool) -> anyhow::Result<()> {
        self.app_config.show_all_players = show_all;
        self.save_config()
    }

    pub fn get_hide_inactive_players(&self) -> bool {
        self.app_config.hide_inactive_players
    }

    pub fn set_hide_inactive_players(&mut self, hide_inactive: bool) -> anyhow::Result<()> {
        self.app_config.hide_inactive_players = hide_inactive;
        self.save_config()
    }

    pub fn get_custom_play_icon(&self) -> Option<String> {
        self.app_config.custom_play_icon.clone()
    }

    pub fn get_custom_pause_icon(&self) -> Option<String> {
        self.app_config.custom_pause_icon.clone()
    }

    pub fn get_custom_next_icon(&self) -> Option<String> {
        self.app_config.custom_next_icon.clone()
    }

    pub fn get_custom_previous_icon(&self) -> Option<String> {
        self.app_config.custom_previous_icon.clone()
    }

    pub fn get_show_controls(&self) -> bool {
        self.app_config.show_controls
    }

    pub fn set_show_controls(&mut self, show_buttons: bool) -> anyhow::Result<()> {
        self.app_config.show_controls = show_buttons;
        self.save_config()
    }

    pub fn get_show_info_pane(&self) -> bool {
        self.app_config.show_info_pane
    }

    pub fn set_show_info_pane(&mut self, show_info_pane: bool) -> anyhow::Result<()> {
        self.app_config.show_info_pane = show_info_pane;
        self.save_config()
    }

    pub fn get_info_pane_left(&self) -> bool {
        self.app_config.info_pane_left
    }

    pub fn set_info_pane_left(&mut self, info_pane_left: bool) -> anyhow::Result<()> {
        self.app_config.info_pane_left = info_pane_left;
        self.save_config()
    }

    pub fn get_info_pane_width(&self) -> u32 {
        self.app_config.info_pane_width
    }

    pub fn set_info_pane_width(&mut self, info_pane_width: u32) -> anyhow::Result<()> {
        self.app_config.info_pane_width = info_pane_width;
        self.save_config()
    }

    pub fn get_album_size(&self) -> u32 {
        self.app_config.album_size
    }

    pub fn set_album_size(&mut self, size: u32) -> anyhow::Result<()> {
        self.app_config.album_size = size;
        self.save_config()
    }


    fn save_config(&self) -> anyhow::Result<()> {
        self.config.set("config", &self.app_config)?;
        Ok(())
    }
}
