use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelsResponse {
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Channel {
    pub id: String,
    pub title: String,
    pub description: String,
    pub dj: String,
    #[serde(default)]
    pub djmail: Option<String>,
    pub genre: String,
    #[serde(deserialize_with = "de_listeners", default)]
    pub listeners: u32,
    #[serde(rename = "lastPlaying", default)]
    pub last_playing: Option<String>,
    pub playlists: Vec<Playlist>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Playlist {
    pub url: String,
    pub format: String,
    pub quality: String,
}

impl Channel {
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

pub fn fetch_channels() -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
    let resp: ChannelsResponse =
        reqwest::blocking::get("https://api.somafm.com/channels.json")?.json()?;
    let mut channels = resp.channels;
    channels.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(channels)
}
