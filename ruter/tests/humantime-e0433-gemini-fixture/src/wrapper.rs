use std::str::FromStr;
use std::ops::Deref;
use std::fmt;
use std::time::{Duration as StdDuration, SystemTime};

use crate::duration::{self, parse_duration, format_duration};
use crate::date::{self, parse_rfc3339_weak, format_rfc3339};

/// A wrapper for duration that has `FromStr` implementation
///
/// This is useful if you want to use it somewhere where `FromStr` is
/// expected.
///
/// See `parse_duration` for the description of the format.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// let x: Duration;
/// x = "12h 5min 2ns".parse::<humantime::Duration>().unwrap().into();
/// assert_eq!(x, Duration::new(12*3600 + 5*60, 2))
/// ```
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Duration(StdDuration);

/// A wrapper for SystemTime that has `FromStr` implementation
///
/// This is useful if you want to use it somewhere where `FromStr` is
/// expected.
///
/// See `parse_rfc3339_weak` for the description of the format. The "weak"
/// format is used as it's more pemissive for human input as this is the
/// expected use of the type (e.g. command-line parsing).
///
/// # Example
///
/// ```
/// use std::time::SystemTime;
/// let x: SystemTime;
/// x = "2018-02-16T00:31:37Z".parse::<humantime::Timestamp>().unwrap().into();
/// assert_eq!(humantime::format_rfc3339(x).to_string(), "2018-02-16T00:31:37Z");
/// ```
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timestamp(SystemTime);

impl AsRef<StdDuration> for Duration {
    fn as_ref(&self) -> &StdDuration { &self.0 }
}

impl Deref for Duration {
    type Target = StdDuration;
    fn deref(&self) -> &StdDuration { &self.0 }
}

impl Into<StdDuration> for Duration {
    fn into(self) -> StdDuration { self.0 }
}

impl From<StdDuration> for Duration {
    fn from(dur: StdDuration) -> Duration { Duration(dur) }
}

impl FromStr for Duration {
    type Err = duration::Error;
    fn from_str(s: &str) -> Result<Duration, Self::Err> {
        parse_duration(s).map(Duration)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_duration(self.0).fmt(f)
    }
}

impl AsRef<SystemTime> for Timestamp {
    fn as_ref(&self) -> &SystemTime { &self.0 }
}

impl Deref for Timestamp {
    type Target = SystemTime;
    fn deref(&self) -> &SystemTime { &self.0 }
}

impl Into<SystemTime> for Timestamp {
    fn into(self) -> SystemTime { self.0 }
}

impl From<SystemTime> for Timestamp {
    fn from(dur: SystemTime) -> Timestamp { Timestamp(dur) }
}

impl FromStr for Timestamp {
    type Err = date::Error;
    fn from_str(s: &str) -> Result<Timestamp, Self::Err> {
        parse_rfc3339_weak(s).map(Timestamp)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_rfc3339(self.0).fmt(f)
    }
}

// >>> E0433 AUTO-INJECTED TESTS BEGIN >>>

// case=24 func_key=<wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref section=parameter_instantiation attempt=1 origin=src/wrapper.rs:110:23

#[cfg(test)]
mod tests_prepare {
    use super::super::humantime_gemini_2_5_flash_nothinking_20251109_134926::src::wrapper::Duration;
    use std::time::Duration as StdDuration; // Alias to avoid confusion

    #[test]
    fn sample() {
        let mut v7 = Duration::from(StdDuration::new(3600, 0)); // create the local variable v7 with type Duration
    }
}

// case=25 func_key=<wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref section=parameter_instantiation attempt=2 origin=src/wrapper.rs:110:23

#[cfg(test)]
mod tests_prepare__case_25 {
    use super::super::humantime_gemini_2_5_flash_nothinking_20251109_134926::wrapper::Duration;
    use std::str::FromStr;

    #[test]
    fn sample() {
        let v7 = Duration::from_str("12h 5min 2ns").unwrap();
    }
}

// case=26 func_key=<wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref section=test_generation attempt=1 origin=src/wrapper.rs:111:16

#[cfg(test)]
mod tests_rug_23 {
    use super::*;
    use crate::std::convert::AsRef;
    use crate::Duration;
    use std::str::FromStr;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_rug() {
        let mut p0: Duration = Duration::from_str("1h 30m 15s").unwrap();

        let std_duration_ref: &StdDuration = <wrapper::Duration>::as_ref(&p0);

        assert_eq!(std_duration_ref, &StdDuration::new(5415, 0));
    }
}

// case=27 func_key=<wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref section=test_generation attempt=2 origin=src/wrapper.rs:111:16

#[cfg(test)]
mod tests_rug_23__case_27 {
    use super::*;
    use crate::std::convert::AsRef;
    use crate::Duration;
    use std::str::FromStr;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_rug() {
        let mut p0: Duration = Duration::from_str("1h 30m 15s").unwrap();

        let std_duration_ref: &StdDuration = <wrapper::Duration>::as_ref(&p0);

        assert_eq!(*std_duration_ref, StdDuration::new(5415, 0));
    }
}

// case=28 func_key=<wrapper::Duration as std::ops::Deref>::deref section=test_generation attempt=1 origin=src/wrapper.rs:127:16

#[cfg(test)]
mod tests_rug_24 {
    use super::*;
    use crate::std::ops::Deref;
    use crate::Duration;
    use std::str::FromStr;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_rug() {
        let mut p0: Duration = Duration::from_str("1h 30m 15s").unwrap();

        let dereferenced_duration: &StdDuration = <wrapper::Duration>::deref(&p0);
        assert_eq!(*dereferenced_duration, StdDuration::new(5415, 0));
    }
}

// // case=29 func_key=<wrapper::Duration as std::ops::Deref>::deref section=test_generation attempt=2 origin=src/wrapper.rs:134:10

// #[cfg(test)]
// mod tests_rug_24__case_29 {
//     use super::*;
//     use std::ops::Deref;
//     use std::str::FromStr;

//     #[test]
//     fn test_rug() {
//         let mut p0: Duration = Duration::from_str("1h 30m 15s").unwrap();

//         <wrapper::Duration>::deref(&p0);
//     }
// }

// // case=30 func_key=<wrapper::Duration as std::ops::Deref>::deref section=test_generation attempt=3 origin=src/wrapper.rs:136:52

// #[cfg(test)]
// mod tests_rug_24__case_30 {
//     use super::*;
//     use crate::wrapper::Duration;
//     use std::ops::Deref;
//     use std::str::FromStr;
//     use std::time::Duration as StdDuration;

//     #[test]
//     fn test_rug() {
//         let mut p0: Duration = Duration::from_str("1h 30m 15s").unwrap();

//         let dereferenced_duration: &StdDuration = <wrapper::Duration>::deref(&p0);
//         assert_eq!(*dereferenced_duration, StdDuration::new(5415, 0));
//     }
// }

// // case=31 func_key=<wrapper::Duration as std::convert::From<std::time::Duration>>::from section=test_generation attempt=1 origin=src/wrapper.rs:163:10

// #[cfg(test)]
// mod tests_rug_26 {
//     use super::*;
//     use std::convert::From;
//     use std::time::Duration as StdDuration;
//     #[test]
//     fn test_rug() {
//         let mut p0: StdDuration = StdDuration::from_secs(120);
//         <wrapper::Duration>::from(p0);
//     }
// }

// // case=32 func_key=<wrapper::Duration as std::str::FromStr>::from_str section=test_generation attempt=1 origin=src/wrapper.rs:174:10

// #[cfg(test)]
// mod tests_rug_27 {
//     use super::*;
//     use std::str::FromStr;
//     #[test]
//     fn test_rug() {
//         let mut p0: &str = "1h 30min 15s";


//         <wrapper::Duration>::from_str(&p0).unwrap();

//     }
// }

// // case=33 func_key=<wrapper::Duration as std::str::FromStr>::from_str section=test_generation attempt=2 origin=src/wrapper.rs:172:10

// #[cfg(test)]
// mod tests_rug_27__case_33 {
//     use super::*;
//     use std::str::FromStr;
//     #[test]
//     fn test_rug() {
//         let mut p0: &str = "1h 30m";
//         <wrapper::Duration>::from_str(&p0).unwrap();
//     }
// }

// // case=34 func_key=<wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref section=test_generation attempt=1 origin=src/wrapper.rs:205:45

// #[cfg(test)]
// mod tests_rug_28 {
//     use super::*;
//     use crate::wrapper::Timestamp;
//     use std::convert::AsRef;
//     use std::str::FromStr;
//     use std::time::SystemTime;

//     #[test]
//     fn test_rug() {
//         let mut p0: Timestamp = Timestamp::from_str("2023-10-27T10:00:00Z").unwrap();

//         let system_time_ref: &SystemTime = <wrapper::Timestamp as AsRef<SystemTime>>::as_ref(&p0);
//         // Add assertions here to verify the returned SystemTime if needed.
//         // For example, you might want to convert `p0` to `SystemTime` directly
//         // and compare it with `system_time_ref`.
//         let expected_system_time: SystemTime = p0.0;
//         assert_eq!(system_time_ref, &expected_system_time);
//     }
// }

// // case=35 func_key=<wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref section=test_generation attempt=2 origin=src/wrapper.rs:217:72

// #[cfg(test)]
// mod tests_rug_28__case_35 {
//     use super::*;
//     use crate::wrapper::Timestamp;
//     use std::convert::AsRef;
//     use std::str::FromStr;
//     use std::time::SystemTime;

//     #[test]
//     fn test_rug() {
//         let p0 = Timestamp::from_str("2023-10-27T10:00:00Z").unwrap();

//         let system_time_ref: &SystemTime = p0.as_ref();

//         // You would typically assert something about the `system_time_ref` here.
//         // For example, if you knew the expected SystemTime, you could compare them.
//         // Since `as_ref` just returns a reference to an internal field, a basic check
//         // would be to ensure it's not null (which is guaranteed by Rust's type system)
//         // or if you have a way to construct the expected SystemTime.

//         // Example: If we assume the internal SystemTime is directly related to the input string.
//         // This requires parsing the string into a SystemTime directly, which humantime already does.
//         // For a more robust test, you'd check if the returned SystemTime matches what
//         // you'd expect from the "2023-10-27T10:00:00Z" string.
//         let expected_time: SystemTime = "2023-10-27T10:00:00Z".parse::<humantime::Timestamp>().unwrap().into();

//         assert_eq!(*system_time_ref, expected_time);
//     }
// }

// // case=36 func_key=<wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into section=test_generation attempt=1 origin=src/wrapper.rs:233:16

// #[cfg(test)]
// mod tests_rug_30 {
//     use super::*;
//     use crate::std::convert::Into;
//     use crate::wrapper::Timestamp;
//     use std::str::FromStr;
//     use std::time::{SystemTime, UNIX_EPOCH};

//     #[test]
//     fn test_rug() {
//         let mut p0: Timestamp = Timestamp::from_str("2023-10-27T10:00:00Z").unwrap();

//         let system_time: SystemTime = <wrapper::Timestamp as Into<SystemTime>>::into(p0);

//         // You might want to add assertions here to verify the conversion.
//         // For example, convert the expected date to a SystemTime and compare.
//         let expected_system_time = UNIX_EPOCH + std::time::Duration::from_secs(1698391200); // "2023-10-27T10:00:00Z"
//         assert_eq!(system_time, expected_system_time);
//     }
// }

// // case=37 func_key=<wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into section=test_generation attempt=2 origin=src/wrapper.rs:242:35

// #[cfg(test)]
// mod tests_rug_30__case_37 {
//     use super::*;
//     use crate::wrapper::Timestamp;
//     use std::convert::Into;
//     use std::str::FromStr;
//     use std::time::{SystemTime, UNIX_EPOCH};

//     #[test]
//     fn test_rug() {
//         let mut p0: Timestamp = Timestamp::from_str("2023-10-27T10:00:00Z").unwrap();

//         let result: SystemTime = <wrapper::Timestamp as Into<SystemTime>>::into(p0);

//         // Assert that the conversion is correct.
//         // The timestamp "2023-10-27T10:00:00Z" corresponds to 1698391200 seconds past the epoch.
//         let expected_system_time = UNIX_EPOCH + std::time::Duration::from_secs(1698391200);
//         assert_eq!(result, expected_system_time);
//     }
// }

// // case=38 func_key=<wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into section=test_generation attempt=3 origin=src/wrapper.rs:242:35

// #[cfg(test)]
// mod tests_rug_30__case_38 {
//     use super::*;
//     use crate::wrapper::Timestamp;
//     use std::convert::Into;
//     use std::str::FromStr;
//     use std::time::{SystemTime, UNIX_EPOCH};

//     #[test]
//     fn test_rug() {
//         let mut p0: Timestamp = Timestamp::from_str("2023-10-27T10:00:00Z").unwrap();

//         let result: SystemTime = <wrapper::Timestamp as Into<SystemTime>>::into(p0);

//         // Assert that the conversion is correct.
//         // The timestamp "2023-10-27T10:00:00Z" corresponds to 1698391200 seconds past UNIX EPOCH.
//         let expected_system_time = UNIX_EPOCH + std::time::Duration::from_secs(1698391200);
//         assert_eq!(result, expected_system_time);
//     }
// }

// // case=39 func_key=<wrapper::Timestamp as std::str::FromStr>::from_str section=test_generation attempt=1 origin=src/wrapper.rs:266:18

// #[cfg(test)]
// mod tests_rug_32 {
//     use super::*;
//     use std::str::FromStr; // Corrected import path

//     #[test]
//     fn test_rug() {
//         let p0: &str = "2023-10-27T10:00:00Z"; 

//         let _ = <wrapper::Timestamp as FromStr>::from_str(p0);
//     }

//     #[test]
//     fn test_rug_invalid_format() {
//         let p0: &str = "invalid-date-format";
//         let result = <wrapper::Timestamp as FromStr>::from_str(p0);
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_rug_rfc3339_full() {
//         let p0: &str = "2023-10-27T10:00:00.123456789Z";
//         let result = <wrapper::Timestamp as FromStr>::from_str(p0);
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_rug_rfc3339_no_seconds() {
//         let p0: &str = "2023-10-27T10:00Z";
//         let result = <wrapper::Timestamp as FromStr>::from_str(p0);
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_rug_rfc3339_with_offset() {
//         let p0: &str = "2023-10-27T10:00:00+01:00";
//         let result = <wrapper::Timestamp as FromStr>::from_str(p0);
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_rug_empty_string() {
//         let p0: &str = "";
//         let result = <wrapper::Timestamp as FromStr>::from_str(p0);
//         assert!(result.is_err());
//     }
// }

// // case=40 func_key=<wrapper::Timestamp as std::str::FromStr>::from_str section=test_generation attempt=2 origin=src/wrapper.rs:266:10

// #[cfg(test)]
// mod tests_rug_32__case_40 {
//     use super::*;
//     use std::str::FromStr;
//     #[test]
//     fn test_rug() {
//         let mut p0: &str = "2023-10-27T10:00:00Z";


//         <wrapper::Timestamp>::from_str(&p0).unwrap();
//         <wrapper::Timestamp>::from_str("2023-10-27T10:00:00.123Z").unwrap();
//         <wrapper::Timestamp>::from_str("2023-10-27T10:00:00-05:00").unwrap();
//         <wrapper::Timestamp>::from_str("2023-10-27T10:00:00").unwrap_err(); // Missing timezone
//         <wrapper::Timestamp>::from_str("invalid_datetime").unwrap_err(); // Invalid format
//     }
// }

// // <<< E0433 AUTO-INJECTED TESTS END <<<
