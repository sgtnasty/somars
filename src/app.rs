use crate::api::Channel;
use crate::player::Player;
use ratatui::widgets::ListState;

pub struct App {
    pub channels: Vec<Channel>,
    pub list_state: ListState,
    pub player: Player,
    pub status_msg: String,
    pub now_playing_idx: Option<usize>,
    pub error: Option<String>,
}

impl App {
    pub fn new(channels: Vec<Channel>) -> Self {
        let mut list_state = ListState::default();
        if !channels.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            channels,
            list_state,
            player: Player::new(),
            status_msg: String::new(),
            now_playing_idx: None,
            error: None,
        }
    }

    pub fn next(&mut self) {
        if self.channels.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.channels.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        if self.channels.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.checked_sub(1).unwrap_or(self.channels.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn play_selected(&mut self) {
        let Some(idx) = self.list_state.selected() else {
            return;
        };
        let Some(url) = self.channels[idx].best_stream_url() else {
            self.error = Some("No stream available for this station".into());
            return;
        };
        let url = url.to_string();
        match self.player.play(&url) {
            Ok(_) => {
                self.now_playing_idx = Some(idx);
                self.status_msg = format!("Playing: {}", self.channels[idx].title);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to start mpv: {} — is mpv installed?", e));
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        if !self.player.is_playing() {
            return;
        }
        self.player.toggle_pause();
        if let Some(i) = self.now_playing_idx {
            let title = self.channels[i].title.clone();
            if self.player.paused {
                self.status_msg = format!("Paused: {}", title);
            } else {
                self.status_msg = format!("Playing: {}", title);
            }
        }
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.now_playing_idx = None;
        self.status_msg = "Stopped".into();
    }

    pub fn volume_up(&mut self) {
        let vol = (self.player.volume + 5).min(100);
        self.player.set_volume(vol);
    }

    pub fn volume_down(&mut self) {
        let vol = self.player.volume.saturating_sub(5);
        self.player.set_volume(vol);
    }

    pub fn tick(&mut self) {
        self.player.check_alive();
        if !self.player.is_playing() && self.now_playing_idx.is_some() {
            self.now_playing_idx = None;
            self.status_msg = "Stream ended".into();
        }
    }

    pub fn selected_channel(&self) -> Option<&Channel> {
        self.list_state
            .selected()
            .and_then(|i| self.channels.get(i))
    }

    pub fn playback_state(&self) -> PlaybackState {
        if !self.player.is_playing() {
            PlaybackState::Stopped
        } else if self.player.paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        }
    }
}

#[derive(PartialEq)]
pub enum PlaybackState {
    Stopped,
    Paused,
    Playing,
}
