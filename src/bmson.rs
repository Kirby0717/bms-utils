use serde::{Deserialize, Serialize};

impl std::str::FromStr for Bmson {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
impl Bmson {
    /// 文字列からBmsonを解析
    pub fn parse(source: &str) -> serde_json::Result<Bmson> {
        serde_json::from_str(source)
    }
    /// BmsonからJson形式の文字列に変換
    pub fn to_json_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
    /// BmsonからJson形式の文字列に変換
    ///
    /// 改行やインデントが適切にされている
    pub fn to_json_string_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

/// Bmson本体
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Bmson {
    /// Bmsonのバージョン
    pub version: String,
    /// 譜面の情報
    pub info: BmsonInfo,
    /// 小節線
    pub lines: Option<Vec<BarLine>>,
    /// BPMイベント
    pub bpm_events: Option<Vec<BpmEvent>>,
    /// 譜面停止イベント
    pub stop_events: Option<Vec<StopEvent>>,
    /// 音声チャンネル
    pub sound_channels: Option<Vec<SoundChannel>>,
    /// BGA情報
    pub bga: Bga,
    /// スクロール速度イベント（beatoraja拡張）
    pub scroll_events: Option<Vec<ScrollEvent>>,
    /// 地雷チャンネル（beatoraja拡張）
    pub mine_channels: Option<Vec<MineChannel>>,
    /// 不可視ノートチャンネル（beatoraja拡張）
    pub key_channels: Option<Vec<KeyChannel>>,
}

/// ヘッダー
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BmsonInfo {
    /// タイトル
    pub title: String,
    /// サブタイトル
    #[serde(default)]
    pub subtitle: String,
    /// アーティスト
    pub artist: String,
    /// サブアーティスト
    pub subartists: Option<Vec<String>>,
    /// ジャンル
    pub genre: String,
    /// プレイ方法のヒント
    ///
    /// beat-7k・popn-5k・generic-nkeysなど
    #[serde(default = "default_mode_hint")]
    pub mode_hint: String,
    /// 難易度
    ///
    /// HYPER・FOUR DIMENSIONSなど
    pub chart_name: String,
    /// レベル
    pub level: u32,
    /// 初期BPM
    ///
    /// 曲選択画面の表示などにも使う
    pub init_bpm: f64,
    /// 判定幅
    ///
    /// 100を初期値とする
    #[serde(default = "default_judge_rank")]
    pub judge_rank: f64,
    /// ゲージ増加の総数
    ///
    /// 全て最良判定のときのゲージの増加量
    ///
    /// 某ゲームでは、総ノーツ数をnとしたとき、7.605 * n / (0.01 * n + 6.5) となる
    #[serde(default = "default_total")]
    pub total: f64,
    /// 背景画像
    pub back_image: Option<String>,
    /// アイキャッチ画像
    pub eyecatch_image: Option<String>,
    /// タイトル画像
    pub title_image: Option<String>,
    /// バナー画像
    pub banner_image: Option<String>,
    /// プレビュー音声
    pub preview_music: Option<String>,
    /// 分解能
    ///
    /// 四分音符1つに対応するパルス数
    ///
    /// 240が初期値
    #[serde(default = "default_resolution")]
    pub resolution: u32,
    /// ロングノートの種類（beatoraja拡張）
    pub ln_type: Option<LongNoteType>,
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
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BarLine {
    /// イベント時刻（パルス数）
    pub y: u32,
}

/// サウンドチャンネル
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SoundChannel {
    /// ファイル名
    pub name: String,
    /// ノーツ
    pub notes: Vec<Note>,
}

/// サウンドノート
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Note {
    /// 演奏レーン
    ///
    /// 0 or NullでBGM
    pub x: Option<u32>,
    /// 演奏時刻（パルス数）
    pub y: u32,
    /// 長さ（パルス数）
    ///
    /// 0で普通のノート
    pub l: u32,
    /// 続行フラグ
    ///
    /// trueでそのまま、falseで音声の最初へもどす
    pub c: bool,
    /// ロングノートの種類（beatoraja拡張）
    pub t: Option<LongNoteType>,
    /// 終端フラグ（beatoraja拡張）
    ///
    /// trueでかつロングノートの終点に配置される場合、終端音として鳴らす
    pub up: Option<bool>,
}

/// BPM変化イベント
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BpmEvent {
    /// イベント時刻（パルス数）
    pub y: u32,
    /// BPM
    pub bpm: f64,
}

/// 譜面停止イベント
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct StopEvent {
    /// イベント時刻（パルス数）
    pub y: u32,
    /// 停止時間（パルス数）
    pub duration: u32,
}

/// BGAデータ
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Bga {
    /// 画像データ
    pub bga_header: Vec<BgaHeader>,
    /// BGAイベント
    pub bga_events: Vec<BgaEvent>,
    /// レイヤーイベント
    pub layer_events: Vec<BgaEvent>,
    /// POORイベント
    pub poor_events: Vec<BgaEvent>,
}

/// 画像ファイル
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BgaHeader {
    /// 画像のID
    pub id: u32,
    /// 画像ファイル
    pub name: String,
}

/// BGAイベント
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BgaEvent {
    /// イベント時刻（パルス数）
    pub y: u32,
    /// 画像のID
    pub id: u32,
}

/// ロングノートの種類（beatoraja拡張）
#[derive(
    serde_repr::Deserialize_repr,
    serde_repr::Serialize_repr,
    Clone,
    Debug,
    PartialEq,
)]
#[repr(u8)]
pub enum LongNoteType {
    /// 未設定
    None = 0,
    /// ロングノート
    ///
    /// 始点と終点の判定の悪い方の判定になる
    ///
    /// ただし、終点のLATE側の判定を除く
    LongNote = 1,
    /// チャージノート
    ///
    /// 始点と終点の両方の判定がある
    ///
    /// 始点を押さなかった場合、終点は見逃したことになる
    ChargeNote = 2,
    /// ヘルチャージノート
    ///
    /// チャージノートに加え、押している間にも16分間隔でロングノート終点と同じ判定が加わる
    ///
    /// 16分間隔の判定は、一度離すと再度押すまで始点の判定になる
    ///
    /// ヘルチャージノーツの終点は普通の判定と同じ
    HellChargeNote = 3,
}

/// スクロール速度設定イベント（beatoraja拡張）
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ScrollEvent {
    /// イベント時刻（パルス数）
    pub y: f64,
    /// スクロール速度倍率
    pub rate: f64,
}

/// 地雷チャンネル（beatoraja拡張）
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct MineChannel {
    /// ファイル名
    pub name: String,
    /// ノーツ
    pub notes: Vec<MineNote>,
}

/// 地雷ノート（beatoraja拡張）
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct MineNote {
    /// 演奏レーン
    ///
    /// 0 or NullでBGM
    pub x: Option<u32>,
    /// 設置時刻（パルス数）
    pub y: u32,
    /// ダメージ
    ///
    /// %で指定
    pub damage: f64,
}

/// 不可視チャンネル（beatoraja拡張）
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct KeyChannel {
    /// ファイル名
    pub name: String,
    /// ノーツ
    pub notes: Vec<KeyNote>,
}

/// 不可視ノート（beatoraja拡張）
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct KeyNote {
    /// 演奏レーン
    ///
    /// 0 or NullでBGM
    pub x: Option<u32>,
    /// 演奏時刻（パルス数）
    pub y: u32,
}
