use cosmic_config::{Config, ConfigGet, ConfigSet};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CONFIG_VERSION: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled_players: HashSet<String>,
    pub auto_detect_new_players: bool,
    pub selected_player: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled_players: HashSet::new(),
            auto_detect_new_players: true,
            selected_player: None,
        }
    }
}

pub struct ConfigManager {
    config: Config,
    app_config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::new("com.cosmic.MusicPlayer", CONFIG_VERSION)?;
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

    fn save_config(&self) -> anyhow::Result<()> {
        self.config.set("config", &self.app_config)?;
        Ok(())
    }
}