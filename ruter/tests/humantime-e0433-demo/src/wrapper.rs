use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::time::{Duration as StdDuration, SystemTime};

use crate::date::{self, format_rfc3339, parse_rfc3339_weak};
use crate::duration::{self, format_duration, parse_duration};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(SystemTime);

impl AsRef<StdDuration> for Duration {
    fn as_ref(&self) -> &StdDuration {
        &self.0
    }
}

impl Deref for Duration {
    type Target = StdDuration;
    fn deref(&self) -> &StdDuration {
        &self.0
    }
}

impl From<Duration> for StdDuration {
    fn from(val: Duration) -> Self {
        val.0
    }
}

impl From<StdDuration> for Duration {
    fn from(dur: StdDuration) -> Duration {
        Duration(dur)
    }
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
    fn as_ref(&self) -> &SystemTime {
        &self.0
    }
}

impl Deref for Timestamp {
    type Target = SystemTime;
    fn deref(&self) -> &SystemTime {
        &self.0
    }
}

impl From<Timestamp> for SystemTime {
    fn from(val: Timestamp) -> Self {
        val.0
    }
}

impl From<SystemTime> for Timestamp {
    fn from(dur: SystemTime) -> Timestamp {
        Timestamp(dur)
    }
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

// --- E0433 演示测试：wrapper 类型相关的各种 crate 路径未声明场景 ---

#[cfg(test)]
mod test {
    /// E0433 场景1：通过完整 crate 路径构造 Duration 包装类型
    #[test]
    fn rug_e0433_duration_from_std() {
        let std_dur = std::time::Duration::new(42, 0);
        let ht: humantime::Duration = humantime::Duration::from(std_dur);
        assert_eq!(ht.as_secs(), 42);
    }

    /// E0433 场景2：通过完整 crate 路径将 humantime::Duration 转回 std::time::Duration
    #[test]
    fn rug_e0433_duration_into_std() {
        let ht: humantime::Duration = "15min".parse().unwrap();
        let std_dur: std::time::Duration = ht.into();
        assert_eq!(std_dur.as_secs(), 900);
    }

    /// E0433 场景3：通过完整 crate 路径从字符串解析 Duration
    #[test]
    fn rug_e0433_duration_from_str_full_path() {
        let ht = "2h 30min".parse::<humantime::Duration>().unwrap();
        assert_eq!(ht.as_secs(), 9000);
    }

    /// E0433 场景4：通过完整 crate 路径显示 Duration
    #[test]
    fn rug_e0433_duration_display_full_path() {
        let ht: humantime::Duration = humantime::Duration::from(std::time::Duration::from_secs(3600));
        assert_eq!(ht.to_string(), "1h");
    }

    /// E0433 场景5：通过完整 crate 路径构造 Timestamp 包装类型
    #[test]
    fn rug_e0433_timestamp_from_system_time() {
        let st = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        let ts: humantime::Timestamp = humantime::Timestamp::from(st);
        let back: std::time::SystemTime = ts.into();
        assert_eq!(back, st);
    }

    /// E0433 场景6：通过完整 crate 路径从字符串解析 Timestamp
    #[test]
    fn rug_e0433_timestamp_from_str_full_path() {
        let ts = "2018-02-16T00:31:37Z".parse::<humantime::Timestamp>().unwrap();
        let expected = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_518_740_897);
        assert_eq!(std::time::SystemTime::from(ts), expected);
    }

    /// E0433 场景7：通过完整 crate 路径显示 Timestamp
    #[test]
    fn rug_e0433_timestamp_display_full_path() {
        let st = std::time::UNIX_EPOCH;
        let ts: humantime::Timestamp = humantime::Timestamp::from(st);
        assert_eq!(ts.to_string(), "1970-01-01T00:00:00Z");
    }

    /// E0433 场景8：在同一函数中同时使用 humantime::Duration 和 humantime::Timestamp
    #[test]
    fn rug_e0433_duration_and_timestamp_together() {
        let dur: humantime::Duration = "1h".parse().unwrap();
        let base = std::time::UNIX_EPOCH;
        let later = base + std::time::Duration::from(*dur);
        let ts: humantime::Timestamp = humantime::Timestamp::from(later);
        assert_eq!(ts.to_string(), "1970-01-01T01:00:00Z");
    }

    /// E0433 场景9：Duration 的 Deref 行为通过完整路径触发
    #[test]
    fn rug_e0433_duration_deref_full_path() {
        let ht: humantime::Duration = humantime::Duration::from(std::time::Duration::from_millis(1500));
        // Deref 到 std::time::Duration 后访问 subsec_millis
        assert_eq!(ht.subsec_millis(), 500);
    }

    /// E0433 场景10：在 Vec 中收集 humantime::Duration 对象（完整路径类型参数）
    #[test]
    fn rug_e0433_vec_of_humantime_duration() {
        let inputs = ["1s", "2min", "1h", "1day"];
        let collected: Vec<humantime::Duration> = inputs
            .iter()
            .map(|s| s.parse::<humantime::Duration>().unwrap())
            .collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0].as_secs(), 1);
        assert_eq!(collected[1].as_secs(), 120);
        assert_eq!(collected[2].as_secs(), 3600);
        assert_eq!(collected[3].as_secs(), 86400);
    }
}
