use serde::Deserialize;
use serde_json::Value;

/// Determine the `DateTime<Utc>` from the provided `Duration` since the epoch.
pub fn datetime_utc_from_epoch_duration(
    duration: std::time::Duration,
) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from(std::time::UNIX_EPOCH + duration)
}

/// Deserialize a `String` as the desired type.
pub fn de_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let data = serde::de::Deserialize::deserialize(deserializer)?;

    match data {
        Value::Number(number) => {
            let as_string = number.to_string();
            T::from_str(&as_string).map_err(serde::de::Error::custom)
        }
        Value::String(string) => string.parse::<T>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("Expected a string or a number")),
    }
}

/// Custom deserializer that tries to parse a string as f64.
/// Returns None if the input is null or cannot be parsed.
pub fn de_option_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let opt_str: Option<&str> = Option::deserialize(deserializer)?;
    match opt_str {
        Some(text) => match text.parse::<f64>() {
            Ok(num) => Ok(Some(num)),
            Err(_) => Err(serde::de::Error::custom("Failed to parse string as f64")),
        },
        None => Ok(None),
    }
}

/// Deserialize a `u64` milliseconds value as `DateTime<Utc>`.
pub fn de_u64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    serde::de::Deserialize::deserialize(deserializer).map(|epoch_ms| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms))
    })
}

/// Deserialize a &str "u64" milliseconds value as `DateTime<Utc>`.
pub fn de_str_u64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_ms| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms))
    })
}

/// Deserialize a &str "f64" milliseconds value as `DateTime<Utc>`.
pub fn de_str_f64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_ms: f64| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms as u64))
    })
}

/// Deserialize a &str "f64" seconds value as `DateTime<Utc>`.
pub fn de_str_f64_epoch_s_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de_str(deserializer).map(|epoch_s: f64| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_secs_f64(epoch_s))
    })
}

/// Assists deserialisation of sequences by attempting to extract & parse the next element in the
/// provided sequence.
///
/// A [`serde::de::Error`] is returned if the element does not exist, or it cannot
/// be deserialized into the `Target` type inferred.
///
/// Example sequence: ["20180.30000","0.00010000","1661978265.280067","s","l",""]
pub fn extract_next<'de, SeqAccessor, Target>(
    sequence: &mut SeqAccessor,
    name: &'static str,
) -> Result<Target, SeqAccessor::Error>
where
    SeqAccessor: serde::de::SeqAccess<'de>,
    Target: serde::de::DeserializeOwned,
{
    sequence
        .next_element::<Target>()?
        .ok_or_else(|| serde::de::Error::missing_field(name))
}

/// Serialize a generic element T as a `Vec<T>`.
pub fn se_element_to_vector<T, S>(element: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    use serde::ser::SerializeSeq;

    let mut sequence = serializer.serialize_seq(Some(1))?;
    sequence.serialize_element(&element)?;
    sequence.end()
}
