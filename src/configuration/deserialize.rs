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
    use hyper::http::uri::Uri;
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
    use hyper::Method;
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
