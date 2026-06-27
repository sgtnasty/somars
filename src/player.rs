use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub struct Player {
    child: Option<Child>,
    socket_path: String,
    pub paused: bool,
    pub volume: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            child: None,
            socket_path: format!("/tmp/somars-{}.sock", std::process::id()),
            paused: false,
            volume: 80,
        }
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol;
        if self.is_playing() {
            let cmd = format!(r#"{{"command":["set_property","volume",{}]}}"#, vol);
            self.send_ipc(&cmd);
        }
    }

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

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_file(&self.socket_path);
        self.paused = false;
    }

    pub fn is_playing(&self) -> bool {
        self.child.is_some()
    }

    /// Check if mpv subprocess has exited and clean up if so.
    pub fn check_alive(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
                self.paused = false;
                let _ = std::fs::remove_file(&self.socket_path);
            }
        }
    }

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
