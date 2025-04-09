//! BMSのライブラリです。
//!
//! BMSファイルの読み書きが出来ます
//!
//! # 拡張子がbms,bme,bml,pmsのファイル
//! ```
//! // 読み込み
//!
//! let bms_str = r"
//! #PLAYER 1
//! #GENRE ジャンル
//! #TITLE タイトル
//! #ARTIST 制作者
//! #BPM 180
//! #PLAYLEVEL 12
//! #RANK 3
//!
//! #SUBTITLE サブタイトル
//! #SUBARTIST サブ制作者
//! #STAGEFILE ステージ画像
//! #BANNER バナー画像
//! #BACKBMP タイトル文字画像
//!
//! #DIFFICULTY 4
//! #TOTAL 400
//! #LNOBJ ZZ
//! #PREVIEW preview.wav
//! #LNMODE 2
//! ";
//! // ランダム要素を確定していない状態のBMSを作成
//! let rawbms = RawBms::parse(bms_str);
//! // ランダム要素を確定させる
//! // この時、疑似乱数生成器を渡す
//! let bms = rawbms.make_bms(rng);
//!
//! // 書き込み
//!
//! // !!!開発中!!!
//! ```
//! # Bmsonファイル
//! ```
//! // 読み込み
//!
//! let bmson = Bmson::parse(&bmson_string).unwrap();
//!
//! // 書き込み
//!
//! // 改行が無く、小さい長さの文字列へ
//! let bmson_string = bmson.to_string().unwrap();
//!
//! // 改行やインデントがなされ、読みやすい文字列へ
//! let bmson_string = bmson.to_string_pretty().unwrap();
//! ```

/// 拡張子がbms,bme,bml,pmsのファイル
///
/// 参考URL
/// * <https://hitkey.nekokan.dyndns.info/cmdsJP.htm>
/// * <https://bemuse.ninja/project/docs/bms-extensions/>
/// * <https://github.com/exch-bms2/beatoraja/wiki/楽曲製作者向け資料>
/// * <https://docs.google.com/document/u/0/d/e/2PACX-1vTl8zOS3ukl5HpuNsBUlN8rn_ZaNdJSHb8a4se3Z3ap9Y6UJ1nB8LA3HnxWAk9kMTDp0j9orpg43-tl/pub>
/// * <https://hitkey.nekokan.dyndns.info/bmse_help_full/beat.html>
pub mod bms;
pub use bms::Bms;
pub use bms::RawBms;

/// 新しいやつ
pub mod new_bms;

/// 拡張子がbmsonのファイル
///
/// 参考URL
/// * <https://bmson-spec.readthedocs.io/en/master/doc/index.html>
/// * <https://github.com/exch-bms2/beatoraja/wiki/楽曲製作者向け資料>
pub mod bmson;
pub use bmson::Bmson;
