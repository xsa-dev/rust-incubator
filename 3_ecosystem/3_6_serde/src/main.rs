use std::{error::Error, time::Duration};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

const REQUEST_JSON: &str = include_str!("../request.json");

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ResponseType {
    Success,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Request {
    #[serde(rename = "type")]
    kind: ResponseType,
    stream: Stream,
    gifts: Vec<Gift>,
    debug: DebugInfo,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Stream {
    user_id: Uuid,
    is_private: bool,
    settings: u32,
    shard_url: Url,
    public_tariff: PublicTariff,
    private_tariff: PrivateTariff,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct PublicTariff {
    id: u32,
    price: u32,
    #[serde(with = "humantime_serde")]
    duration: Duration,
    description: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct PrivateTariff {
    client_price: u32,
    #[serde(with = "humantime_serde")]
    duration: Duration,
    description: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Gift {
    id: u32,
    price: u32,
    description: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DebugInfo {
    #[serde(with = "humantime_serde")]
    duration: Duration,
    at: DateTime<FixedOffset>,
}

fn parse_request() -> Result<Request, serde_json::Error> {
    serde_json::from_str(REQUEST_JSON)
}

fn convert_request() -> Result<(String, String), Box<dyn Error>> {
    let request = parse_request()?;
    let yaml = serde_yaml::to_string(&request)?;
    let toml = toml::to_string_pretty(&request)?;
    Ok((yaml, toml))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (yaml, toml) = convert_request()?;
    println!("{}", yaml.trim_end());
    println!("{}", toml.trim_end());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_deserialized_correctly() {
        let request = parse_request().expect("request to be parsed");

        assert_eq!(request.kind, ResponseType::Success);
        assert_eq!(
            request.stream.user_id,
            Uuid::parse_str("8d234120-0bda-49b2-b7e0-fbd3912f6cbf").unwrap()
        );
        assert_eq!(request.stream.is_private, false);
        assert_eq!(request.stream.settings, 45_345);
        assert_eq!(
            request.stream.shard_url,
            Url::parse("https://n3.example.com/sapi").unwrap()
        );
        assert_eq!(request.stream.public_tariff.id, 1);
        assert_eq!(request.stream.public_tariff.price, 100);
        assert_eq!(
            request.stream.public_tariff.duration,
            Duration::from_secs(3600)
        );
        assert_eq!(
            request.stream.public_tariff.description,
            "test public tariff"
        );
        assert_eq!(request.stream.private_tariff.client_price, 250);
        assert_eq!(
            request.stream.private_tariff.duration,
            Duration::from_secs(60)
        );
        assert_eq!(
            request.stream.private_tariff.description,
            "test private tariff"
        );

        assert_eq!(request.gifts.len(), 2);
        assert_eq!(request.gifts[0].id, 1);
        assert_eq!(request.gifts[0].price, 2);
        assert_eq!(request.gifts[0].description, "Gift 1");
        assert_eq!(request.gifts[1].id, 2);
        assert_eq!(request.gifts[1].price, 3);
        assert_eq!(request.gifts[1].description, "Gift 2");

        assert_eq!(request.debug.duration, Duration::from_millis(234));
        assert_eq!(
            request.debug.at,
            DateTime::parse_from_rfc3339("2019-06-28T08:35:46+00:00").unwrap()
        );
    }

    #[test]
    fn serializations_are_correct() {
        let (yaml, toml) = convert_request().expect("conversion to succeed");

        let request = parse_request().expect("request to be parsed");

        let yaml_parsed: Request = serde_yaml::from_str(&yaml).expect("YAML to deserialize");
        assert_eq!(yaml_parsed, request);

        let toml_parsed: Request = toml::from_str(&toml).expect("TOML to deserialize");
        assert_eq!(toml_parsed, request);
    }
}
