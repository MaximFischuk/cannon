use core::str::FromStr;
use regex::Regex;
use lazy_static::*;
use crate::time::error::Error;
use std::time::Duration;

lazy_static! {
    // static ref UNIT_REGEX: Regex = Regex::new(
    //     r"^(?=\d+[ywdhms])(( ?\d+y)?(?!\d))?(( ?\d+w)?(?!\d))?(( ?\d+d)?(?!\d))?(( ?\d+h)?(?!\d))?(( ?\d+m)?(?!\d))?(( ?\d+s)?(?!\d))?( ?\d+ms)?$"
    // )
    // .expect("Regex compilation error");
    
    static ref DURATION_REGEX: Regex = Regex::new(
        r"^(?P<value>\d+)(?P<unit>ns|us|ms|s|m|h|d){1}$"
    )
    .expect("Regex compilation error");
}

pub struct DurationUnit {
    value: u64,
    unit: TimeUnit,
}

#[derive(Debug, PartialEq)]
pub enum TimeUnit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}

impl FromStr for DurationUnit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if DURATION_REGEX.is_match(s) {
            let caps = DURATION_REGEX.captures(s).unwrap();
            let value = caps.name("value").unwrap().as_str().parse().unwrap();
            let time_unit = caps.name("unit").unwrap().as_str();
            let unit = time_unit.parse::<TimeUnit>()?;
            Ok(Self { value, unit })
        } else {
            Err(Error::Syntax("Current string is not correct duration unit value".to_owned()))
        }
    }

}

impl Into<Duration> for DurationUnit {
    
    fn into(self) -> Duration {
        match self.unit {
            TimeUnit::Nanosecond => Duration::from_nanos(self.value),
            TimeUnit::Microsecond => Duration::from_micros(self.value),
            TimeUnit::Millisecond => Duration::from_millis(self.value),
            TimeUnit::Second => Duration::from_secs(self.value),
            TimeUnit::Minute => Duration::from_secs(self.value * 60),
            TimeUnit::Hour => Duration::from_secs(self.value * 60 * 60),
            TimeUnit::Day => Duration::from_secs(self.value * 60 * 60 * 24),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ns" | "nanosecond" | "nanos" | "nanoseconds" => Ok(TimeUnit::Nanosecond),
            "us" | "microsecond" | "micros" | "microseconds" => Ok(TimeUnit::Microsecond),
            "ms" | "millisecond" | "millis" | "milliseconds" => Ok(TimeUnit::Millisecond),
            "s" | "second" | "secs" | "seconds" => Ok(TimeUnit::Second),
            "m" | "minute" | "mins" | "minutes" => Ok(TimeUnit::Minute),
            "h" | "hour" | "hours" => Ok(TimeUnit::Hour),
            "d" | "day" | "days" => Ok(TimeUnit::Day),
            _ => Err(Error::UnitNotSupported(format!("Unit '{}' not supported", s)))
        }
    }

}

#[cfg(test)]
mod tests {
    use std::time::Duration;
use crate::time::timeunit::DurationUnit;
use crate::time::timeunit::TimeUnit;

    #[test]
    fn test_building_time_unit_from_string() {
        {
            let value = "ns";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Nanosecond);
        }
        {
            let value = "us";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Microsecond);
        }
        {
            let value = "ms";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Millisecond);
        }
        {
            let value = "s";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Second);
        }
        {
            let value = "m";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Minute);
        }
        {
            let value = "h";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Hour);
        }
        {
            let value = "d";
            let result = value.parse::<TimeUnit>();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TimeUnit::Day);
        }
    }

    #[test]
    fn test_conversion_duration_unit_to_duration() {
        let value = "200ms";
        let unit = value.parse::<DurationUnit>().unwrap();
        let result: Duration = unit.into();

        assert_eq!(result, Duration::from_millis(200));
    }
}
