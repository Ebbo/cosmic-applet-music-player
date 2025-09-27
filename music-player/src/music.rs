use anyhow::Result;
use mpris::{PlaybackStatus, Player, PlayerFinder};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub title: String,
    pub artist: String,
    pub status: PlaybackStatus,
    pub volume: f64,
    pub art_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPlayer {
    pub identity: String,
    pub is_active: bool,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            title: "No music playing".to_string(),
            artist: String::new(),
            status: PlaybackStatus::Stopped,
            volume: 0.5,
            art_url: None,
        }
    }
}

#[derive(Clone)]
pub struct MusicController {
    player: Arc<Mutex<Option<Player>>>,
    discovered_players: Arc<Mutex<HashMap<String, DiscoveredPlayer>>>,
}

impl MusicController {
    pub fn new() -> Self {
        Self {
            player: Arc::new(Mutex::new(None)),
            discovered_players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn discover_all_players(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        if let Ok(mut discovered_lock) = self.discovered_players.lock() {
            discovered_lock.clear();

            // Try to get all players
            if let Ok(players) = player_finder.find_all() {
                for player in players {
                    let identity = player.identity();
                    let is_active = player.get_playback_status().unwrap_or(PlaybackStatus::Stopped) == PlaybackStatus::Playing;

                    discovered_lock.insert(identity.to_string(), DiscoveredPlayer {
                        identity: identity.to_string(),
                        is_active,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn find_active_player(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // Try to find the first available player
        if let Ok(player) = player_finder.find_active() {
            if let Ok(mut player_lock) = self.player.lock() {
                *player_lock = Some(player);
            }
        }

        Ok(())
    }


    pub fn find_specific_player(&mut self, player_name: &str) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // Try to find all players and pick the one that matches the name
        if let Ok(players) = player_finder.find_all() {
            for player in players {
                let identity = player.identity();
                if identity == player_name {
                    if let Ok(mut player_lock) = self.player.lock() {
                        *player_lock = Some(player);
                        return Ok(());
                    }
                }
            }
        }

        // Player not found, clear current player
        if let Ok(mut player_lock) = self.player.lock() {
            *player_lock = None;
        }

        Ok(())
    }

    pub fn get_discovered_players(&self) -> Vec<DiscoveredPlayer> {
        if let Ok(discovered_lock) = self.discovered_players.lock() {
            discovered_lock.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_player_info(&self) -> PlayerInfo {
        let player_guard = match self.player.lock() {
            Ok(guard) => guard,
            Err(_) => return PlayerInfo::default(),
        };

        let Some(ref player) = *player_guard else {
            return PlayerInfo::default();
        };

        let metadata = player.get_metadata().unwrap_or_default();
        let status = player.get_playback_status().unwrap_or(PlaybackStatus::Stopped);
        let volume = player.get_volume().unwrap_or(0.5);

        let title = metadata
            .title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let artist = metadata
            .artists()
            .map(|artists| artists.join(", "))
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let art_url = metadata.art_url().map(|url| url.to_string());

        PlayerInfo {
            title,
            artist,
            status,
            volume,
            art_url,
        }
    }

    pub fn play_pause(&self) -> Result<()> {
        let player_guard = match self.player.lock() {
            Ok(guard) => guard,
            Err(_) => return Ok(()),
        };

        if let Some(ref player) = *player_guard {
            player.play_pause()?;
        }
        Ok(())
    }

    pub fn next(&self) -> Result<()> {
        let player_guard = match self.player.lock() {
            Ok(guard) => guard,
            Err(_) => return Ok(()),
        };

        if let Some(ref player) = *player_guard {
            player.next()?;
        }
        Ok(())
    }

    pub fn previous(&self) -> Result<()> {
        let player_guard = match self.player.lock() {
            Ok(guard) => guard,
            Err(_) => return Ok(()),
        };

        if let Some(ref player) = *player_guard {
            player.previous()?;
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let player_guard = match self.player.lock() {
            Ok(guard) => guard,
            Err(_) => return Ok(()),
        };

        if let Some(ref player) = *player_guard {
            player.set_volume(volume)?;
        }
        Ok(())
    }
}