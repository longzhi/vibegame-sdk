//! 现在几点：时钟、宠物这种看时间的 app 用。

/// 一个本地时间（设备的时区）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    /// Unix 秒。算过了多久用它：存下来，下次开的时候相减。
    pub unix: i64,
    pub year: i32,
    /// 1 到 12。
    pub month: u32,
    /// 1 到 31。
    pub day: u32,
    /// 0 是星期一，6 是星期日。
    pub weekday: u32,
    /// 0 到 23。
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Time {
    /// 从 Unix 秒和时区偏移（秒）算出本地时间。测试里用它造一个时间。
    pub fn at(unix: i64, tz_offset: i32) -> Time {
        let local = unix + tz_offset as i64;
        let days = local.div_euclid(86_400);
        let secs = local.rem_euclid(86_400) as u32;
        // 公历的年月日（Howard Hinnant 的 civil_from_days）。
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = (yoe + era * 400 + if month <= 2 { 1 } else { 0 }) as i32;
        // 1970-01-01 是星期四。
        let weekday = (days + 3).rem_euclid(7) as u32;
        Time { unix, year, month, day, weekday, hour: secs / 3_600, minute: secs / 60 % 60, second: secs % 60 }
    }
}

/// 现在的本地时间。掌机上是掌机的钟和时区；`vibeplay run --time` 能把钟定住，截图才稳。
pub fn now() -> Time {
    let (unix, tz) = qapp::host::now();
    Time::at(unix, tz)
}

#[cfg(test)]
mod tests {
    use super::Time;

    #[test]
    fn dates_and_weekdays() {
        let t = Time::at(0, 0);
        assert_eq!((t.year, t.month, t.day, t.weekday, t.hour), (1970, 1, 1, 3, 0));
        // 2026-11-14 07:30:05 东京（+9），星期六。
        let t = Time::at(1_794_609_005, 9 * 3600);
        assert_eq!((t.year, t.month, t.day, t.weekday, t.hour, t.minute, t.second), (2026, 11, 14, 5, 7, 30, 5));
        // 闰年的 2 月 29 日。
        let t = Time::at(1_709_164_800, 0);
        assert_eq!((t.year, t.month, t.day), (2024, 2, 29));
        // 时区往回拨跨过零点：UTC 1 月 1 日 01:00，在 -5 是前一年的 12 月 31 日。
        let t = Time::at(1_767_229_200, -5 * 3600);
        assert_eq!((t.year, t.month, t.day, t.hour), (2025, 12, 31, 20));
    }
}
