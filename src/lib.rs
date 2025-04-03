//! BMSのライブラリです。
//!
//! 主にBMSファイルの読み書きが出来ます
//!
//! まだ調整中
//!
//! 要望がある場合はGithubのissueへ

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

/// 拡張子がbmsonのファイル
///
/// 参考URL
/// * <https://bmson-spec.readthedocs.io/en/master/doc/index.html>
/// * <https://github.com/exch-bms2/beatoraja/wiki/楽曲製作者向け資料>
pub mod bmson;
pub use bmson::Bmson;
