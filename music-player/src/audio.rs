use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AudioSinkInput {
    pub index: u32,
    pub application_name: String,
    pub media_name: String,
    pub volume: f64,
}

pub struct AudioController {
    sink_inputs: Arc<Mutex<HashMap<u32, AudioSinkInput>>>,
}

impl AudioController {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sink_inputs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn connect(&self) -> Result<()> {
        // No connection needed for pactl-based approach
        Ok(())
    }

    pub fn refresh_sink_inputs(&self) -> Result<()> {
        // Use pactl to list sink inputs
        let output = Command::new("pactl")
            .arg("list")
            .arg("sink-inputs")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("pactl command failed"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut sink_inputs = self.sink_inputs.lock().unwrap();
        sink_inputs.clear();

        let mut current_index: Option<u32> = None;
        let mut current_app_name = String::new();
        let mut current_media_name = String::new();
        let mut current_volume = 1.0;

        for line in output_str.lines() {
            let line = line.trim();

            if line.starts_with("Sink Input #") {
                // Save previous entry if exists
                if let Some(index) = current_index {
                    sink_inputs.insert(
                        index,
                        AudioSinkInput {
                            index,
                            application_name: current_app_name.clone(),
                            media_name: current_media_name.clone(),
                            volume: current_volume,
                        },
                    );
                }

                // Start new entry
                if let Some(index_str) = line.strip_prefix("Sink Input #") {
                    current_index = index_str.parse().ok();
                    current_app_name = String::new();
                    current_media_name = String::new();
                    current_volume = 1.0;
                }
            } else if line.starts_with("application.name = ") {
                current_app_name = line
                    .strip_prefix("application.name = \"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap_or("")
                    .to_string();
            } else if line.starts_with("media.name = ") {
                current_media_name = line
                    .strip_prefix("media.name = \"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap_or("")
                    .to_string();
            } else if line.starts_with("Volume:") {
                // Parse volume percentage (e.g., "Volume: front-left: 65536 / 100%")
                if let Some(percent_pos) = line.find('%') {
                    let before_percent = &line[..percent_pos];
                    if let Some(last_space) = before_percent.rfind(' ') {
                        if let Ok(percent) = before_percent[last_space + 1..].trim().parse::<f64>() {
                            current_volume = percent / 100.0;
                        }
                    }
                }
            }
        }

        // Save last entry
        if let Some(index) = current_index {
            sink_inputs.insert(
                index,
                AudioSinkInput {
                    index,
                    application_name: current_app_name,
                    media_name: current_media_name,
                    volume: current_volume,
                },
            );
        }

        Ok(())
    }

    pub fn find_sink_input_by_name(&self, app_name_pattern: &str) -> Option<AudioSinkInput> {
        let sink_inputs = self.sink_inputs.lock().unwrap();

        let pattern_lower = app_name_pattern.to_lowercase();

        for sink_input in sink_inputs.values() {
            let app_name_lower = sink_input.application_name.to_lowercase();
            if app_name_lower.contains(&pattern_lower) || pattern_lower.contains(&app_name_lower) {
                return Some(sink_input.clone());
            }
        }

        None
    }

    pub fn set_sink_input_volume(&self, index: u32, volume: f64) -> Result<()> {
        // Clamp volume to 0.0-1.5 (150%)
        let clamped_volume = volume.clamp(0.0, 1.5);

        // Convert to percentage
        let volume_percent = (clamped_volume * 100.0) as u32;

        // Use pactl to set volume
        let output = Command::new("pactl")
            .arg("set-sink-input-volume")
            .arg(index.to_string())
            .arg(format!("{}%", volume_percent))
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("pactl set-sink-input-volume failed"));
        }

        Ok(())
    }

    pub fn get_all_sink_inputs(&self) -> Vec<AudioSinkInput> {
        let sink_inputs = self.sink_inputs.lock().unwrap();
        sink_inputs.values().cloned().collect()
    }
}

impl Drop for AudioController {
    fn drop(&mut self) {
        // No cleanup needed for pactl-based approach
    }
}
