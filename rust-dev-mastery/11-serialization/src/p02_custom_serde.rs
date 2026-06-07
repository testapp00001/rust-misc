//! # Custom Serialization/Deserialization
//!
//! When derive macros aren't enough, you can implement `Serialize` and
//! `Deserialize` manually. This gives full control over the wire format.
//!
//! Key patterns:
//! - `Visitor` pattern for deserialization
//! - `DeserializeSeed` for stateful deserialization
//! - Custom `Serializer`/`Deserializer` for new formats
//! - Converting between representations

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;

/// Demonstrates custom Serialize/Deserialize for a type with a non-standard format.
/// `Fraction` serializes as "numerator/denominator" string.
#[derive(Debug, Clone, PartialEq)]
pub struct Fraction {
    pub numerator: i64,
    pub denominator: i64,
}

impl Serialize for Fraction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}/{}", self.numerator, self.denominator);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Fraction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FractionVisitor)
    }
}

struct FractionVisitor;

impl<'de> Visitor<'de> for FractionVisitor {
    type Value = Fraction;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a fraction string like '3/4'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Fraction, E>
    where
        E: de::Error,
    {
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() != 2 {
            return Err(E::custom(format!("invalid fraction: {value}")));
        }
        let numerator = parts[0].parse().map_err(E::custom)?;
        let denominator = parts[1].parse().map_err(E::custom)?;
        if denominator == 0 {
            return Err(E::custom("denominator cannot be zero"));
        }
        Ok(Fraction {
            numerator,
            denominator,
        })
    }
}

/// Demonstrates custom serialization with multiple acceptable formats.
/// `FlexibleDate` can deserialize from either a string or a structured object.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexibleDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Serialize for FlexibleDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:04}-{:02}-{:02}", self.year, self.month, self.day);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for FlexibleDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FlexibleDateVisitor)
    }
}

struct FlexibleDateVisitor;

impl<'de> Visitor<'de> for FlexibleDateVisitor {
    type Value = FlexibleDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a date string 'YYYY-MM-DD' or object {year, month, day}")
    }

    fn visit_str<E>(self, value: &str) -> Result<FlexibleDate, E>
    where
        E: de::Error,
    {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Err(E::custom(format!("invalid date format: {value}")));
        }
        Ok(FlexibleDate {
            year: parts[0].parse().map_err(E::custom)?,
            month: parts[1].parse().map_err(E::custom)?,
            day: parts[2].parse().map_err(E::custom)?,
        })
    }

    fn visit_map<M>(self, mut map: M) -> Result<FlexibleDate, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut year = None;
        let mut month = None;
        let mut day = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "year" => year = Some(map.next_value()?),
                "month" => month = Some(map.next_value()?),
                "day" => day = Some(map.next_value()?),
                _ => {
                    let _ = map.next_value::<serde_json::Value>()?;
                }
            }
        }

        Ok(FlexibleDate {
            year: year.ok_or_else(|| de::Error::missing_field("year"))?,
            month: month.ok_or_else(|| de::Error::missing_field("month"))?,
            day: day.ok_or_else(|| de::Error::missing_field("day"))?,
        })
    }
}

/// Demonstrates custom serialization for a newtype with validation.
#[derive(Debug, Clone, PartialEq)]
pub struct Email(pub String);

impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if !s.contains('@') {
            return Err(de::Error::custom("invalid email: missing @"));
        }
        Ok(Email(s))
    }
}

/// Demonstrates custom struct serialization with flattened fields.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiResponse<T> {
    pub status: u16,
    pub message: String,
    pub data: Option<T>,
    pub errors: Vec<String>,
}

impl<T: Serialize> Serialize for ApiResponse<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 3 + if self.data.is_some() { 1 } else { 0 };
        let mut state = serializer.serialize_struct("ApiResponse", field_count)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("message", &self.message)?;
        if let Some(ref data) = self.data {
            state.serialize_field("data", data)?;
        }
        state.serialize_field("errors", &self.errors)?;
        state.end()
    }
}

impl<'de, T: serde::de::DeserializeOwned> Deserialize<'de> for ApiResponse<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use a concrete helper to avoid type inference issues
        #[derive(serde::Deserialize)]
        struct Helper {
            status: u16,
            message: String,
            #[serde(default)]
            data: Option<serde_json::Value>,
            #[serde(default)]
            errors: Vec<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let data: Option<T> = helper.data.and_then(|v| serde_json::from_value(v).ok());

        Ok(ApiResponse {
            status: helper.status,
            message: helper.message,
            data,
            errors: helper.errors,
        })
    }
}

/// Demonstrates serializing a type as a sequence of bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct HexBytes(pub Vec<u8>);

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = self.0.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() % 2 != 0 {
            return Err(de::Error::custom("hex string must have even length"));
        }
        let bytes = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(de::Error::custom))
            .collect::<Result<Vec<u8>, _>>()?;
        Ok(HexBytes(bytes))
    }
}

/// Demonstrates custom serialization for a set (order-independent equality).
#[derive(Debug, Clone, PartialEq)]
pub struct StringSet {
    items: Vec<String>,
}

impl StringSet {
    pub fn new(items: Vec<String>) -> Self {
        let mut items = items;
        items.sort();
        items.dedup();
        StringSet { items }
    }

    pub fn contains(&self, item: &str) -> bool {
        self.items.binary_search_by(|s| s.as_str().cmp(item)).is_ok()
    }
}

impl Serialize for StringSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.items.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StringSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items = Vec::<String>::deserialize(deserializer)?;
        Ok(StringSet::new(items))
    }
}

/// Demonstrates a visitor that handles multiple input types.
#[derive(Debug, Clone, PartialEq)]
pub enum Nullable<T> {
    Null,
    Value(T),
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Nullable::Null => serializer.serialize_none(),
            Nullable::Value(v) => serializer.serialize_some(v),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NullableVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for NullableVisitor<T> {
            type Value = Nullable<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("null or a value")
            }

            fn visit_none<E>(self) -> Result<Nullable<T>, E>
            where
                E: de::Error,
            {
                Ok(Nullable::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Nullable<T>, D::Error>
            where
                D: Deserializer<'de>,
            {
                T::deserialize(deserializer).map(Nullable::Value)
            }

            fn visit_unit<E>(self) -> Result<Nullable<T>, E>
            where
                E: de::Error,
            {
                Ok(Nullable::Null)
            }
        }

        deserializer.deserialize_option(NullableVisitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_serialize() {
        let f = Fraction {
            numerator: 3,
            denominator: 4,
        };
        let json = serde_json::to_string(&f).unwrap();
        assert_eq!(json, "\"3/4\"");
    }

    #[test]
    fn test_fraction_deserialize() {
        let f: Fraction = serde_json::from_str("\"3/4\"").unwrap();
        assert_eq!(f.numerator, 3);
        assert_eq!(f.denominator, 4);
    }

    #[test]
    fn test_fraction_invalid() {
        assert!(serde_json::from_str::<Fraction>("\"3/0\"").is_err());
        assert!(serde_json::from_str::<Fraction>("\"abc\"").is_err());
    }

    #[test]
    fn test_flexible_date_from_string() {
        let d: FlexibleDate = serde_json::from_str("\"2024-01-15\"").unwrap();
        assert_eq!(d.year, 2024);
        assert_eq!(d.month, 1);
        assert_eq!(d.day, 15);
    }

    #[test]
    fn test_flexible_date_from_object() {
        let d: FlexibleDate =
            serde_json::from_str(r#"{"year": 2024, "month": 1, "day": 15}"#).unwrap();
        assert_eq!(d.year, 2024);
    }

    #[test]
    fn test_email_validation() {
        assert!(serde_json::from_str::<Email>("\"user@example.com\"").is_ok());
        assert!(serde_json::from_str::<Email>("\"invalid\"").is_err());
    }

    #[test]
    fn test_api_response() {
        let resp = ApiResponse {
            status: 200,
            message: "ok".to_string(),
            data: Some(42),
            errors: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("200"));
        assert!(json.contains("data"));

        let back: ApiResponse<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.data, Some(42));
    }

    #[test]
    fn test_api_response_no_data() {
        let resp: ApiResponse<()> = ApiResponse {
            status: 404,
            message: "not found".to_string(),
            data: None,
            errors: vec!["resource missing".to_string()],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_hex_bytes() {
        let bytes = HexBytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let json = serde_json::to_string(&bytes).unwrap();
        assert_eq!(json, "\"deadbeef\"");

        let back: HexBytes = serde_json::from_str("\"deadbeef\"").unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn test_string_set() {
        let set = StringSet::new(vec![
            "c".to_string(),
            "a".to_string(),
            "b".to_string(),
            "a".to_string(), // duplicate
        ]);
        let json = serde_json::to_string(&set).unwrap();
        assert_eq!(json, "[\"a\",\"b\",\"c\"]");

        let back: StringSet = serde_json::from_str(&json).unwrap();
        assert_eq!(back, set);
        assert!(back.contains("a"));
        assert!(!back.contains("d"));
    }

    #[test]
    fn test_nullable() {
        let n: Nullable<i32> = Nullable::Value(42);
        let json = serde_json::to_string(&n).unwrap();
        assert_eq!(json, "42");

        let n: Nullable<i32> = Nullable::Null;
        let json = serde_json::to_string(&n).unwrap();
        assert_eq!(json, "null");

        let back: Nullable<i32> = serde_json::from_str("null").unwrap();
        assert_eq!(back, Nullable::Null);
    }
}
