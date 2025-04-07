use serde::{Deserialize, Serialize};
use serde_with::*;
use serde_json::Result;

/// Bmson本体
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Bmson {
    pub version: String,
    pub info: BmsonInfo,
    pub lines: Option<Vec<BarLine>>,
    pub bpm_events: Option<Vec<BpmEvent>>,
    pub stop_events: Option<Vec<StopEvent>>,
    pub sound_channels: Option<Vec<SoundChannel>>,
    pub bga: Bga,
    /// beatoraja拡張
    pub scroll_events: Option<Vec<ScrollEvent>>,
    /// beatoraja拡張
    pub mine_channels: Option<Vec<MineChannel>>,
    /// beatoraja拡張
    pub key_channels: Option<Vec<KeyChannel>>,
}

/// ヘッダー
#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BmsonInfo {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub artist: String,
    pub subartists: Option<Vec<String>>,
    pub genre: String,
    #[serde(default = "default_mode_hint")]
    pub mode_hint: String,
    pub chart_name: String,
    pub level: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub init_bpm: f64,
    #[serde(default = "default_judge_rank")]
    pub judge_rank: f64,
    #[serde(default = "default_total")]
    pub total: f64,
    pub back_image: Option<String>,
    pub eyecatch_image: Option<String>,
    pub banner_image: Option<String>,
    pub preview_music: Option<String>,
    #[serde(default = "default_resolution")]
    pub resolution: u32,
    /// beatoraja拡張
    pub ln_type: Option<String>,
}
fn default_mode_hint() -> String {
    "beat-7k".to_string()
}
fn default_judge_rank() -> f64 {
    100.
}
fn default_total() -> f64 {
    100.
}
fn default_resolution() -> u32 {
    240
}

/// 小節線イベント
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BarLine {
    pub y: u32,
}

/// サウンドチャンネル
///
/// DTMソフトのトラックと似ている
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct SoundChannel {
    pub name: String,
    pub notes: Vec<Note>,
}

/// サウンドノート
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Note {
    /// 演奏レーン
    ///
    /// 0 or NullでGBM
    pub x: Option<u32>,
    pub y: u32,
    pub l: u32,
    pub c: bool,
    /// beatoraja拡張
    pub t: Option<String>,
    /// beatoraja拡張
    pub up: Option<bool>,
}

/// BPM変化イベント
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BpmEvent {
    pub y: u32,
    pub bpm: f64,
}

/// 譜面停止イベント
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StopEvent {
    pub y: u32,
    pub duration: u32,
}

/// BGAデータ
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Bga {
    pub bga_header: Vec<BgaHeader>,
    pub bga_events: Vec<BgaEvent>,
    pub layer_events: Vec<BgaEvent>,
    pub poor_events: Vec<BgaEvent>,
}

/// 画像ファイル
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BgaHeader {
    pub id: u32,
    pub name: String,
}

/// BGA設定イベント
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BgaEvent {
    pub y: u32,
    pub id: u32,
}

/// スクロール速度設定イベント（beatoraja拡張）
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ScrollEvent {
    pub y: f64,
}

/// 地雷チャンネル（beatoraja拡張）
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct MineChannel {
    pub name: String,
    pub notes: Vec<MineNote>,
}

/// 地雷ノート（beatoraja拡張）
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct MineNote {
    pub x: Option<u32>,
    pub y: u32,
    pub damage: f64,
}

/// 不可視チャンネル（beatoraja拡張）
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct KeyChannel {
    pub name: String,
    pub notes: Vec<KeyNote>,
}

/// 不可視ノート（beatoraja拡張）
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct KeyNote {
    pub x: Option<u32>,
    pub y: u32,
}

impl std::str::FromStr for Bmson {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
impl std::fmt::Display for Bmson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self)
                .map_err(|_| { std::fmt::Error })?
        )?;
        Ok(())
    }
}
impl Bmson {
    pub fn parse(source: &str) -> Result<Bmson> {
        serde_json::from_str(source)
    }
}
