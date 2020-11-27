use std::fmt;
use time::{
    PrimitiveDateTime, OffsetDateTime, UtcOffset, Date,
};
use serde::{
    Deserialize, Serialize, Deserializer, Serializer,
    de::{
        self, Visitor
    }
};

const INVALID_DATE_TIME: OffsetDateTime = time::date!(0-01-01).midnight().assume_utc();

#[inline]
pub(crate) fn parse_z_offset_date_time(s: impl AsRef<str>, format: impl AsRef<str>) -> time::Result<OffsetDateTime> {
    let format = String::from(format.as_ref()).replace("%Z", "Z");
    Ok(PrimitiveDateTime::parse(s, format.as_str())?.assume_utc())
}

#[derive(Debug, PartialEq)]
pub struct UtcDateTime {
    inner: OffsetDateTime
}

impl UtcDateTime {
    pub fn is_valid(&self) -> bool {
        self.inner != INVALID_DATE_TIME
    }
    
    pub fn as_ref(&self) -> &OffsetDateTime {
        &self.inner
    }
}

impl Into<OffsetDateTime> for UtcDateTime {
    fn into(self) -> OffsetDateTime {
        self.inner
    }
}

impl Into<UtcDateTime> for OffsetDateTime {
    fn into(self) -> UtcDateTime {
        UtcDateTime {
            inner: self.to_offset(UtcOffset::UTC),
        }
    }
}

impl Into<UtcDateTime> for PrimitiveDateTime {
    fn into(self) -> UtcDateTime {
        UtcDateTime {
            inner: self.assume_utc(),
        }
    }
}

impl Into<UtcDateTime> for Date {
    fn into(self) -> UtcDateTime {
        UtcDateTime {
            inner: self.midnight().assume_utc(),
        }
    }
}

impl ToString for UtcDateTime {
    fn to_string(&self) -> String {
        self.inner.format("%Y-%m-%dT%H:%M:%SZ")
    }
}

impl Into<UtcDateTime> for &str {
    fn into(self) -> UtcDateTime {
        let inner_date_time = if let Ok(date_time) = parse_z_offset_date_time(self, "%Y-%m-%dT%H:%M:%S%Z") {
            date_time
        } else {
            INVALID_DATE_TIME
        };

        inner_date_time.into()
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D>(deserializer: D) -> Result<UtcDateTime, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct UtcDateTimeVisitor;

        impl<'de> Visitor<'de> for UtcDateTimeVisitor {
            type Value = UtcDateTime;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<UtcDateTime, E>
                where
                    E: de::Error,
            {
                let date_time: UtcDateTime = value.into();
                Ok(date_time)
            }
        }

        deserializer.deserialize_string(UtcDateTimeVisitor)
    }
}

impl Serialize for UtcDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

mod tests {
    use super::*;

    #[test]
    fn it_convert_from_offset_date_time() {
        let offset_date_time = time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc();
        let date_time: UtcDateTime = offset_date_time.into();

        assert_eq!(date_time.is_valid(), true);
        assert_eq!(date_time, UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        });
    }

    #[test]
    fn it_convert_from_primitive_date_time() {
        let primitive_date_time = time::date!(2020-11-27).with_time(time::time!(07:53:25));
        let date_time: UtcDateTime = primitive_date_time.into();

        assert_eq!(date_time.is_valid(), true);
        assert_eq!(date_time, UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        });
    }

    #[test]
    fn it_convert_from_date() {
        let date = time::date!(2020-11-27);
        let date_time: UtcDateTime = date.into();

        assert_eq!(date_time.is_valid(), true);
        assert_eq!(date_time, UtcDateTime {
            inner: time::date!(2020-11-27).midnight().assume_utc()
        });
    }

    #[test]
    fn it_convert_from_string() {
        let date_time: UtcDateTime = "2020-11-27T07:53:25Z".into();

        assert_eq!(date_time.is_valid(), true);
        assert_eq!(date_time, UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        });
    }

    #[test]
    fn it_convert_from_invalid_string() {
        let date_time: UtcDateTime = "bla".into();

        assert_eq!(date_time.is_valid(), false);
        assert_eq!(date_time, UtcDateTime {
            inner: INVALID_DATE_TIME
        });
    }

    #[test]
    fn it_convert_to_offset_date_time() {
        let date_time = UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        };
        let offset_date_time: OffsetDateTime = date_time.into();

        assert_eq!(offset_date_time, time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc());
    }

    #[test]
    fn it_format_to_string() {
        let date_time = UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        };
        let dt_str = date_time.to_string();

        assert_eq!(dt_str, "2020-11-27T07:53:25Z");
    }

    #[test]
    fn it_get_reference() {
        let date_time = UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        };

        assert_eq!(date_time.as_ref(), &date_time.inner);
    }

    #[test]
    fn it_deserialize_date_time() {
        let json_str = r#""2020-11-27T07:53:25Z""#;

        let date_time: UtcDateTime = serde_json::from_str(json_str).unwrap();

        assert_eq!(date_time, UtcDateTime {
            inner: time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc()
        });
    }

    #[test]
    fn it_serialize_date_time() {
        let date_time: UtcDateTime = time::date!(2020-11-27).with_time(time::time!(07:53:25)).assume_utc().into();

        let json_str = serde_json::to_string(&date_time).unwrap();

        assert_eq!(json_str, r#""2020-11-27T07:53:25Z""#);
    }
}