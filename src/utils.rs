use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn random_percentage() -> u32 {
    rand::random_range(0..100)
}

pub fn parse_json_data<T: DeserializeOwned + Default>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap_or_default()
}
