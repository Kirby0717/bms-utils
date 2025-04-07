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

/// ハッシュや同値判定ができる少数
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Decimal {
    /// 整数部分
    pub integer_part: i64,
    /// 小数部分
    pub fractional_part: u64,
}

impl From<Decimal> for f32 {
    fn from(value: Decimal) -> Self {
        let fractional_part = if value.fractional_part == 0 {
            0.0
        }
        else {
            value.fractional_part as f32
                * 10.0_f32.powi(-(1 + value.fractional_part.ilog10() as i32))
        };
        if 0 <= value.integer_part {
            value.integer_part as f32 + fractional_part
        }
        else {
            value.integer_part as f32 - fractional_part
        }
    }
}
impl From<Decimal> for f64 {
    fn from(value: Decimal) -> Self {
        let fractional_part = if value.fractional_part == 0 {
            0.0
        }
        else {
            value.fractional_part as f64
                * 10.0_f64.powi(-(1 + value.fractional_part.ilog10() as i32))
        };
        if 0 <= value.integer_part {
            value.integer_part as f64 + fractional_part
        }
        else {
            value.integer_part as f64 - fractional_part
        }
    }
}
impl serde_with::SerializeAs<f64> for Decimal {
    fn serialize_as<S>(source: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        //serializer.serialize_str(s)
        todo!()
    }
}

impl<'de> serde_with::DeserializeAs<'de, f64> for Decimal {
    fn deserialize_as<D>(deserializer: D) -> Result<f64, D::Error>
        where
            D: serde::Deserializer<'de> {
        //let s = String::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        todo!()
    }
}
