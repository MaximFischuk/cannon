pub mod selector {
    use jsonpath::Selector;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Selector, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|v| Selector::new(v.as_str()).unwrap())
    }
}

pub mod uri {
    use http::Uri;
    use serde::{Deserialize, Deserializer};
    use std::str::FromStr;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|v| Uri::from_str(v.as_str()).unwrap())
    }
}

pub mod http_method {
    use reqwest::Method;
    use serde::{Deserialize, Deserializer};
    use std::str::FromStr;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Method, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|v| Method::from_str(v.as_str()).unwrap())
    }
}

pub mod base64_property {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|v| base64::decode(v).unwrap())
    }
}

pub mod duration {
    use crate::time::timeunit::DurationUnit;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer};
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(v) => {
                let value = match v.as_str().parse::<DurationUnit>() {
                    Ok(value) => value.into(),
                    Err(err) => return Err(D::Error::custom(err.to_string())),
                };
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }
}
