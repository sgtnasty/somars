use crate::api::Channel;
use crate::player::Player;
use ratatui::widgets::ListState;

/// Central application state, updated on every input event and tick.
pub struct App {
    /// Full list of SomaFM stations fetched at startup, sorted alphabetically.
    pub channels: Vec<Channel>,
    /// ratatui list cursor. Always indexes into the *filtered* view, not `channels` directly.
    pub list_state: ListState,
    /// Audio player managing the mpv subprocess.
    pub player: Player,
    /// Text shown in the left portion of the status bar.
    pub status_msg: String,
    /// Index into `channels` of the currently playing station, if any.
    ///
    /// Always refers to the full `channels` list, not the filtered view, so the
    /// playing indicator survives filter changes.
    pub now_playing_idx: Option<usize>,
    /// Live track title polled from the stream's ICY metadata every ~2 seconds.
    ///
    /// `None` until the first successful poll after playback starts.
    pub now_playing_title: Option<String>,
    /// Error message displayed in the detail panel, if any.
    pub error: Option<String>,
    /// Current filter query string, matched against title, genre, and description.
    pub filter: String,
    /// Whether filter input mode is active (typing appends to `filter`).
    pub filter_active: bool,
    pub tick_count: u32,
}

impl App {
    /// Creates a new `App` with the given channel list and selects the first item.
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

    /// Returns the indices into `channels` that match the current filter query.
    ///
    /// Matches case-insensitively against title, genre, and description.
    /// Returns all indices when the filter is empty.
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

    /// Moves the selection down by one in the filtered list, wrapping around.
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

    /// Moves the selection up by one in the filtered list, wrapping around.
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

    /// Plays the currently selected station.
    ///
    /// Resolves the filtered-list selection back to an index in the full `channels`
    /// list, picks the best available stream URL, and delegates to [`Player::play`].
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

    /// Stops playback and clears all playing state.
    pub fn stop(&mut self) {
        self.player.stop();
        self.now_playing_idx = None;
        self.now_playing_title = None;
        self.status_msg = "Stopped".into();
    }

    /// Increases volume by 5%, capped at 100.
    pub fn volume_up(&mut self) {
        let vol = (self.player.volume + 5).min(100);
        self.player.set_volume(vol);
    }

    /// Decreases volume by 5%, floored at 0.
    pub fn volume_down(&mut self) {
        let vol = self.player.volume.saturating_sub(5);
        self.player.set_volume(vol);
    }

    /// Activates filter input mode.
    pub fn enter_filter(&mut self) {
        self.filter_active = true;
    }

    /// Deactivates filter input mode without clearing the query.
    pub fn exit_filter(&mut self) {
        self.filter_active = false;
    }

    /// Appends a character to the filter query and resets the list selection to the top.
    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.reset_selection();
    }

    /// Removes the last character from the filter query and resets the list selection.
    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.reset_selection();
    }

    /// Clears the filter query, deactivates filter mode, and resets the list selection.
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_active = false;
        self.reset_selection();
    }

    fn reset_selection(&mut self) {
        let n = self.filtered_indices().len();
        self.list_state.select(if n == 0 { None } else { Some(0) });
    }

    /// Called every ~100 ms to detect stream changes and poll for live track metadata.
    ///
    /// Queries mpv for the current ICY title every ~2 seconds while a station is playing.
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

    /// Returns a reference to the currently selected [`Channel`], if any.
    pub fn selected_channel(&self) -> Option<&Channel> {
        let filtered = self.filtered_indices();
        self.list_state
            .selected()
            .and_then(|fi| filtered.get(fi))
            .and_then(|&ci| self.channels.get(ci))
    }

    /// Returns the index into `channels` of the currently selected station, if any.
    pub fn selected_channel_idx(&self) -> Option<usize> {
        let filtered = self.filtered_indices();
        self.list_state
            .selected()
            .and_then(|fi| filtered.get(fi))
            .copied()
    }

    /// Returns the current playback state.
    pub fn playback_state(&self) -> PlaybackState {
        if self.player.is_playing() {
            PlaybackState::Playing
        } else {
            PlaybackState::Stopped
        }
    }
}

/// The two possible playback states of the player.
#[derive(PartialEq)]
pub enum PlaybackState {
    /// No mpv subprocess is running.
    Stopped,
    /// mpv is running and actively playing.
    Playing,
}
