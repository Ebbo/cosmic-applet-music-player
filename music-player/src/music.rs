use anyhow::Result;
use mpris::{FindingError, PlaybackStatus, Player, PlayerFinder};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use crate::audio::AudioController;

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub title: String,
    pub artist: String,
    pub status: PlaybackStatus,
    pub volume: f64,
    pub art_url: Option<String>,
    pub bus_name: String,
    pub identity: String,
    pub can_control_volume: bool,
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
            bus_name: String::new(),
            identity: String::new(),
            can_control_volume: true,
        }
    }
}

#[derive(Clone)]
pub struct MusicController {
    player: Rc<RefCell<Option<Player>>>,
    discovered_players: Rc<RefCell<HashMap<String, DiscoveredPlayer>>>,
    all_players: Rc<RefCell<HashMap<String, Player>>>,
    /// Last successful reading per bus name, used to ride out a failed poll in
    /// multi-player mode instead of dropping the player out of the list.
    last_known_info: Rc<RefCell<HashMap<String, PlayerInfo>>>,
    audio_controller: Option<Arc<AudioController>>,
}

impl MusicController {
    pub fn new() -> Self {
        // Try to initialize audio controller, but don't fail if it doesn't work
        let audio_controller = AudioController::new()
            .and_then(|ac| {
                ac.connect()?;
                Ok(Arc::new(ac))
            })
            .ok();

        if audio_controller.is_none() {
            eprintln!("Warning: Failed to initialize PulseAudio/PipeWire audio controller");
        }

        Self {
            player: Rc::new(RefCell::new(None)),
            discovered_players: Rc::new(RefCell::new(HashMap::new())),
            all_players: Rc::new(RefCell::new(HashMap::new())),
            last_known_info: Rc::new(RefCell::new(HashMap::new())),
            audio_controller,
        }
    }

    pub fn discover_all_players(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // `find_all` fails as a whole if any single player on the bus is slow to
        // answer or exits mid-enumeration, so build the new maps aside and swap
        // them in only once the enumeration actually succeeded. Clearing first
        // blanks the panel for a tick every time one unrelated player hiccups.
        let Ok(players) = player_finder.find_all() else {
            return Ok(());
        };

        let mut discovered = HashMap::new();
        let mut all_players = HashMap::new();

        for player in players {
            let identity = player.identity();
            let bus_name = player.bus_name_player_name_part();
            let is_active = player
                .get_playback_status()
                .unwrap_or(PlaybackStatus::Stopped)
                == PlaybackStatus::Playing;

            discovered.insert(
                identity.to_string(),
                DiscoveredPlayer {
                    identity: identity.to_string(),
                    is_active,
                },
            );

            all_players.insert(bus_name.to_string(), player);
        }

        *self.discovered_players.borrow_mut() = discovered;
        *self.all_players.borrow_mut() = all_players;

        Ok(())
    }

    pub fn find_active_player(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        match player_finder.find_active() {
            Ok(player) => *self.player.borrow_mut() = Some(player),
            // The bus answered and there is genuinely nobody there.
            Err(FindingError::NoPlayerFound) => *self.player.borrow_mut() = None,
            // A transient D-Bus failure says nothing about the player we already
            // hold, so keep it instead of falling back to "No music playing".
            Err(FindingError::DBusError(_)) => {}
        }

        Ok(())
    }

    pub fn find_specific_player(&mut self, player_name: &str) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // Same caveat as `discover_all_players`: an `Err` here means the
        // enumeration failed, not that our player went away.
        let Ok(players) = player_finder.find_all() else {
            return Ok(());
        };

        for player in players {
            let identity = player.identity();
            if identity == player_name {
                *self.player.borrow_mut() = Some(player);
                return Ok(());
            }
        }

        // The bus was enumerated successfully and our player was not on it.
        *self.player.borrow_mut() = None;

        Ok(())
    }

    pub fn get_discovered_players(&self) -> Vec<DiscoveredPlayer> {
        self.discovered_players.borrow().values().cloned().collect()
    }

    /// Reads one player's current state.
    ///
    /// Returns `None` when a D-Bus read fails, which means "ask again next
    /// tick" — not "nothing is playing". Callers must not turn that into
    /// default/placeholder values or the panel flickers on every hiccup.
    ///
    /// Assumes the caller has already refreshed the audio controller's sink
    /// inputs, so it can be called in a loop without one refresh per player.
    fn extract_player_info(&self, player: &Player, bus_name: &str) -> Option<PlayerInfo> {
        let metadata = player.get_metadata().ok()?;
        let status = player.get_playback_status().ok()?;

        // Spotify (among others) briefly answers with an empty metadata map
        // while switching tracks. That reads as a success but would publish
        // "Unknown" and drop the album art for a tick, so treat it as a failed
        // read. When playback has genuinely stopped, empty metadata is real.
        if metadata.is_empty() && status != PlaybackStatus::Stopped {
            return None;
        }

        let mut volume = player.get_volume().unwrap_or(0.5);

        let title = metadata
            .title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let artist = metadata
            .artists()
            .map(|artists| artists.join(", "))
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let art_url = metadata.art_url().map(|url| url.to_string());
        let identity = player.identity().to_string();

        // For browsers, get actual volume from PulseAudio
        if let Some(ref audio_ctrl) = self.audio_controller {
            if let Some(sink_input) = audio_ctrl.find_sink_input_by_name(&identity) {
                volume = sink_input.volume;
            }
        }

        Some(PlayerInfo {
            title,
            artist,
            status,
            volume,
            art_url,
            bus_name: bus_name.to_string(),
            identity,
            // Volume control is now supported for all players
            // MPRIS-supporting players use MPRIS, browsers use PulseAudio/PipeWire fallback
            can_control_volume: true,
        })
    }

    /// `Some` is a fresh reading to display. `None` means this poll failed and
    /// the caller should keep showing what it already has.
    pub fn get_player_info(&self) -> Option<PlayerInfo> {
        let player_borrow = self.player.borrow();

        let Some(ref player) = *player_borrow else {
            return Some(PlayerInfo::default());
        };

        if let Some(ref audio_ctrl) = self.audio_controller {
            let _ = audio_ctrl.refresh_sink_inputs();
        }

        let bus_name = player.bus_name_player_name_part().to_string();
        self.extract_player_info(player, &bus_name)
    }

    pub fn get_all_players_info(&self) -> Vec<PlayerInfo> {
        let all_players_borrow = self.all_players.borrow();
        let mut last_known = self.last_known_info.borrow_mut();
        let mut players_info: Vec<PlayerInfo> = Vec::new();
        let mut firefox_players: Vec<PlayerInfo> = Vec::new();

        // Refresh audio controller sink inputs if available
        if let Some(ref audio_ctrl) = self.audio_controller {
            let _ = audio_ctrl.refresh_sink_inputs();
        }

        // Drop cached readings for players that have left the bus.
        last_known.retain(|bus_name, _| all_players_borrow.contains_key(bus_name));

        for (bus_name, player) in all_players_borrow.iter() {
            let player_info = match self.extract_player_info(player, bus_name) {
                Some(info) => {
                    last_known.insert(bus_name.clone(), info.clone());
                    info
                }
                // Reuse the last good reading so the row neither disappears nor
                // resets to "Unknown" for a single tick.
                None => match last_known.get(bus_name) {
                    Some(info) => info.clone(),
                    None => continue,
                },
            };

            let identity = player_info.identity.clone();

            // Separate Firefox players for deduplication
            if identity.to_lowercase().contains("firefox") {
                firefox_players.push(player_info);
            } else {
                players_info.push(player_info);
            }
        }

        // Deduplicate Firefox: keep only the most relevant one (Playing > Paused > Stopped)
        if !firefox_players.is_empty() {
            // Sort Firefox players by status priority
            firefox_players.sort_by(|a, b| {
                let status_order = |status: &PlaybackStatus| match status {
                    PlaybackStatus::Playing => 0,
                    PlaybackStatus::Paused => 1,
                    PlaybackStatus::Stopped => 2,
                };
                status_order(&a.status).cmp(&status_order(&b.status))
            });

            // Take the first one (most relevant)
            if let Some(firefox_player) = firefox_players.into_iter().next() {
                players_info.push(firefox_player);
            }
        }

        // Sort players by identity for stable ordering (alphabetical)
        // This prevents players from jumping around when status changes
        players_info.sort_by(|a, b| {
            a.identity.to_lowercase().cmp(&b.identity.to_lowercase())
        });

        players_info
    }

    pub fn play_pause_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.play_pause()?;
        }
        Ok(())
    }

    pub fn next_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.next()?;
        }
        Ok(())
    }

    pub fn previous_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.previous()?;
        }
        Ok(())
    }

    pub fn set_volume_player(&self, bus_name: &str, volume: f64) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();

        if let Some(player) = all_players_borrow.get(bus_name) {
            // Try MPRIS first
            if let Ok(_) = player.set_volume(volume) {
                return Ok(());
            }

            // If MPRIS fails, try audio controller (for browsers)
            if let Some(ref audio_ctrl) = self.audio_controller {
                let identity = player.identity();

                // First refresh to get current sink inputs
                let _ = audio_ctrl.refresh_sink_inputs();

                // Try to find matching audio stream
                if let Some(sink_input) = audio_ctrl.find_sink_input_by_name(identity) {
                    audio_ctrl.set_sink_input_volume(sink_input.index, volume)?;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn play_pause(&self) -> Result<()> {
        let player_borrow = self.player.borrow();

        if let Some(ref player) = *player_borrow {
            player.play_pause()?;
        }
        Ok(())
    }

    pub fn next(&self) -> Result<()> {
        let player_borrow = self.player.borrow();

        if let Some(ref player) = *player_borrow {
            player.next()?;
        }
        Ok(())
    }

    pub fn previous(&self) -> Result<()> {
        let player_borrow = self.player.borrow();

        if let Some(ref player) = *player_borrow {
            player.previous()?;
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let player_borrow = self.player.borrow();

        if let Some(ref player) = *player_borrow {
            // Try MPRIS first
            if let Ok(_) = player.set_volume(volume) {
                return Ok(());
            }

            // If MPRIS fails, try audio controller (for browsers)
            if let Some(ref audio_ctrl) = self.audio_controller {
                let identity = player.identity();

                // First refresh to get current sink inputs
                let _ = audio_ctrl.refresh_sink_inputs();

                // Try to find matching audio stream
                if let Some(sink_input) = audio_ctrl.find_sink_input_by_name(identity) {
                    audio_ctrl.set_sink_input_volume(sink_input.index, volume)?;
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
