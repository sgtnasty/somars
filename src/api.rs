use serde::Deserialize;

/// Raw JSON response envelope from the SomaFM channels endpoint.
#[derive(Debug, Deserialize, Clone)]
pub struct ChannelsResponse {
    /// All available stations.
    pub channels: Vec<Channel>,
}

/// A single SomaFM radio station with its metadata and available stream playlists.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Channel {
    /// Unique station identifier used in SomaFM URLs (e.g. `"groovesalad"`).
    pub id: String,
    /// Human-readable station name (e.g. `"Groove Salad"`).
    pub title: String,
    /// Short description of the station's genre and vibe.
    pub description: String,
    /// Name of the station's DJ or host.
    pub dj: String,
    /// DJ contact email, if provided.
    #[serde(default)]
    pub djmail: Option<String>,
    /// Genre tag (e.g. `"ambient"`, `"electronic"`).
    pub genre: String,
    /// Current live listener count. Deserialized from either a number or a numeric string.
    #[serde(deserialize_with = "de_listeners", default)]
    pub listeners: u32,
    /// Most recently played track from the initial API response, in `"Artist - Title"` format.
    ///
    /// Shown in the detail panel until live ICY metadata is available.
    #[serde(rename = "lastPlaying", default)]
    pub last_playing: Option<String>,
    /// Available stream formats and quality levels.
    pub playlists: Vec<Playlist>,
}

/// A single stream variant for a [`Channel`], identified by format and quality.
#[derive(Debug, Deserialize, Clone)]
pub struct Playlist {
    /// URL of the `.pls` playlist file. mpv resolves this to the actual stream URL natively.
    pub url: String,
    /// Stream codec (`"mp3"` or `"aac"`).
    pub format: String,
    /// Quality tier: `"highest"`, `"high"`, or `"low"`.
    pub quality: String,
}

impl Channel {
    /// Returns the URL of the best available stream playlist for this station.
    ///
    /// Quality priority order: `highest` → `high` → `low`. Returns `None` if the
    /// channel has no playlists.
    pub fn best_stream_url(&self) -> Option<&str> {
        let quality_rank = |q: &str| match q {
            "highest" => 0,
            "high" => 1,
            "low" => 2,
            _ => 3,
        };
        self.playlists
            .iter()
            .min_by_key(|p| quality_rank(p.quality.as_str()))
            .map(|p| p.url.as_str())
    }

    /// Returns the best available [`Playlist`] entry for this station.
    ///
    /// Quality priority order: `highest` → `high` → `low`. Returns `None` if the
    /// channel has no playlists.
    pub fn best_playlist(&self) -> Option<&Playlist> {
        let quality_rank = |q: &str| match q {
            "highest" => 0,
            "high" => 1,
            "low" => 2,
            _ => 3,
        };
        self.playlists
            .iter()
            .min_by_key(|p| quality_rank(p.quality.as_str()))
    }
}

fn de_listeners<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = u32;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number or numeric string")
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<u32, E> {
            Ok(v as u32)
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<u32, E> {
            Ok(v.max(0) as u32)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<u32, E> {
            v.parse().map_err(E::custom)
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<u32, E> {
            v.parse().map_err(E::custom)
        }
    }
    d.deserialize_any(V)
}

/// Fetches all SomaFM channels from the API and returns them sorted alphabetically by title.
///
/// Makes a blocking HTTP GET request to `https://api.somafm.com/channels.json`.
///
/// # Errors
///
/// Returns an error if the network request fails or the response cannot be deserialized.
pub fn fetch_channels() -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
    let resp: ChannelsResponse =
        reqwest::blocking::get("https://api.somafm.com/channels.json")?.json()?;
    let mut channels = resp.channels;
    channels.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(channels)
}
