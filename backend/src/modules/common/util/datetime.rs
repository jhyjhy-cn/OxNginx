use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serializer};

/// ponytail: NaiveDateTime 序列化为 "YYYY-MM-DD HH:MM:SS"（chrono Display 默认格式）
/// 配套 #[serde(with = "naive_datetime")] 在 entity 字段上使用
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub mod naive_datetime {
    use super::*;

    pub fn serialize<S: Serializer>(dt: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&dt.format(FORMAT).to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<NaiveDateTime, D::Error> {
        let s = String::deserialize(d)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

/// Option<NaiveDateTime> 版本
pub mod option_naive_datetime {
    use super::*;

    pub fn serialize<S: Serializer>(dt: &Option<NaiveDateTime>, s: S) -> Result<S::Ok, S::Error> {
        match dt {
            Some(v) => s.serialize_str(&v.format(FORMAT).to_string()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<NaiveDateTime>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        match opt {
            Some(s) => NaiveDateTime::parse_from_str(&s, FORMAT)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}
