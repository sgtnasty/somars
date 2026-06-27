use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Manages an `mpv` subprocess and communicates with it via a UNIX IPC socket.
pub struct Player {
    child: Option<Child>,
    socket_path: String,
    /// Whether playback is currently paused.
    pub paused: bool,
    /// Current volume level (0–100).
    pub volume: u8,
}

impl Player {
    /// Creates a new `Player` with the default volume (80) and no active subprocess.
    ///
    /// The IPC socket path is set to `/tmp/somars-<pid>.sock`.
    pub fn new() -> Self {
        Self {
            child: None,
            socket_path: format!("/tmp/somars-{}.sock", std::process::id()),
            paused: false,
            volume: 80,
        }
    }

    /// Sets the volume (0–100), applying the change immediately if a stream is playing.
    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol;
        if self.is_playing() {
            let cmd = format!(r#"{{"command":["set_property","volume",{}]}}"#, vol);
            self.send_ipc(&cmd);
        }
    }

    /// Spawns `mpv` to play the given stream URL.
    ///
    /// Stops any currently playing stream before starting the new one. mpv is launched
    /// with `--no-video` and an IPC socket at [`Player::socket_path`] for subsequent control.
    ///
    /// # Errors
    ///
    /// Returns an error if `mpv` cannot be spawned.
    pub fn play(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();
        let child = Command::new("mpv")
            .args([
                "--no-video",
                "--no-terminal",
                &format!("--volume={}", self.volume),
                &format!("--input-ipc-server={}", self.socket_path),
                url,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        self.child = Some(child);
        self.paused = false;
        Ok(())
    }

    /// Toggles pause/resume via the mpv IPC socket.
    ///
    /// Has no effect if nothing is playing.
    pub fn toggle_pause(&mut self) {
        if !self.is_playing() {
            return;
        }
        let cmd = if self.paused {
            r#"{"command":["set_property","pause",false]}"#
        } else {
            r#"{"command":["set_property","pause",true]}"#
        };
        if self.send_ipc(cmd) {
            self.paused = !self.paused;
        }
    }

    /// Kills the mpv subprocess and removes the IPC socket file.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_file(&self.socket_path);
        self.paused = false;
    }

    /// Returns `true` if an mpv subprocess is currently running.
    pub fn is_playing(&self) -> bool {
        self.child.is_some()
    }

    /// Checks whether the mpv subprocess has exited and cleans up state if so.
    ///
    /// Called on every tick to detect streams that end on their own.
    pub fn check_alive(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.paused = false;
                let _ = std::fs::remove_file(&self.socket_path);
            }
        }
    }

    /// Queries mpv for the current track title via ICY stream metadata.
    ///
    /// Returns `None` if no stream is playing, the socket is not yet ready,
    /// or no ICY title has been received from the stream.
    pub fn get_media_title(&self) -> Option<String> {
        self.query_ipc(r#"{"command":["get_property","metadata/by-key/icy-title"]}"#)
    }

    fn query_ipc(&self, cmd: &str) -> Option<String> {
        let mut stream = UnixStream::connect(&self.socket_path).ok()?;
        stream.set_write_timeout(Some(Duration::from_millis(100))).ok()?;
        stream.set_read_timeout(Some(Duration::from_millis(100))).ok()?;
        writeln!(stream, "{}", cmd).ok()?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        for _ in 0..10 {
            line.clear();
            if reader.read_line(&mut line).ok()? == 0 {
                return None;
            }
            let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
            if v.get("error").is_some() {
                return if v["error"].as_str() == Some("success") {
                    v["data"].as_str().map(|s| s.to_owned())
                } else {
                    None
                };
            }
        }
        None
    }

    fn send_ipc(&self, cmd: &str) -> bool {
        for _ in 0..5 {
            if let Ok(mut stream) = UnixStream::connect(&self.socket_path) {
                let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
                let _ = writeln!(stream, "{}", cmd);
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}
