use crate::api::Channel;
use crate::player::Player;
use ratatui::widgets::ListState;

pub struct App {
    pub channels: Vec<Channel>,
    pub list_state: ListState,
    pub player: Player,
    pub status_msg: String,
    pub now_playing_idx: Option<usize>,
    pub now_playing_title: Option<String>,
    pub error: Option<String>,
    pub filter: String,
    pub filter_active: bool,
    tick_count: u32,
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
            now_playing_title: None,
            error: None,
            filter: String::new(),
            filter_active: false,
            tick_count: 0,
        }
    }

    /// Indices into self.channels that match the current filter.
    pub fn filtered_indices(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            return (0..self.channels.len()).collect();
        }
        let q = self.filter.to_lowercase();
        self.channels
            .iter()
            .enumerate()
            .filter(|(_, ch)| {
                ch.title.to_lowercase().contains(&q)
                    || ch.genre.to_lowercase().contains(&q)
                    || ch.description.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn next(&mut self) {
        let n = self.filtered_indices().len();
        if n == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % n,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let n = self.filtered_indices().len();
        if n == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.checked_sub(1).unwrap_or(n - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn play_selected(&mut self) {
        let filtered = self.filtered_indices();
        let Some(fi) = self.list_state.selected() else {
            return;
        };
        let Some(&ci) = filtered.get(fi) else {
            return;
        };
        let Some(url) = self.channels[ci].best_stream_url() else {
            self.error = Some("No stream available for this station".into());
            return;
        };
        let url = url.to_string();
        match self.player.play(&url) {
            Ok(_) => {
                self.now_playing_idx = Some(ci);
                self.now_playing_title = None;
                self.status_msg = format!("Playing: {}", self.channels[ci].title);
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
        self.now_playing_title = None;
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

    pub fn enter_filter(&mut self) {
        self.filter_active = true;
    }

    pub fn exit_filter(&mut self) {
        self.filter_active = false;
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.reset_selection();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.reset_selection();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_active = false;
        self.reset_selection();
    }

    fn reset_selection(&mut self) {
        let n = self.filtered_indices().len();
        self.list_state.select(if n == 0 { None } else { Some(0) });
    }

    pub fn tick(&mut self) {
        self.player.check_alive();
        if !self.player.is_playing() && self.now_playing_idx.is_some() {
            self.now_playing_idx = None;
            self.now_playing_title = None;
            self.status_msg = "Stream ended".into();
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.player.is_playing() && self.tick_count % 20 == 0 {
            if let Some(title) = self.player.get_media_title() {
                self.now_playing_title = Some(title);
            }
        }
    }

    pub fn selected_channel(&self) -> Option<&Channel> {
        let filtered = self.filtered_indices();
        self.list_state
            .selected()
            .and_then(|fi| filtered.get(fi))
            .and_then(|&ci| self.channels.get(ci))
    }

    pub fn selected_channel_idx(&self) -> Option<usize> {
        let filtered = self.filtered_indices();
        self.list_state
            .selected()
            .and_then(|fi| filtered.get(fi))
            .copied()
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
