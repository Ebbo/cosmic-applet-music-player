use anyhow::Result;
use mpris::{PlaybackStatus, Player, PlayerFinder};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub title: String,
    pub artist: String,
    pub status: PlaybackStatus,
    pub volume: f64,
    pub art_url: Option<String>,
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
}

impl MusicController {
    pub fn new() -> Self {
        Self {
            player: Arc::new(Mutex::new(None))
        }
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