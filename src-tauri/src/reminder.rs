use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Local, NaiveDate};

use crate::db::ReminderSettings;

/// 计算下一次提醒的等待秒数与触发日期（本地墙钟时间）。
///
/// 若今日设定时刻尚未到达，返回今日；否则返回明日同一时刻。
/// 返回值恒为正（≥ 1 秒），避免忙循环。小时/分钟会被限制在合法范围。
pub fn next_fire(now: &DateTime<Local>, settings: &ReminderSettings) -> (u64, NaiveDate) {
    let hour = settings.hour.clamp(0, 23) as u32;
    let minute = settings.minute.clamp(0, 59) as u32;
    let now_local = now.naive_local();
    let today = now_local.date();

    let target_today = today
        .and_hms_opt(hour, minute, 0)
        .unwrap_or_else(|| today.and_hms_opt(20, 0, 0).expect("20:00 is always valid"));
    let delay = (target_today - now_local).num_seconds();
    if delay > 0 {
        return (delay as u64, today);
    }

    let tomorrow = today + chrono::Days::new(1);
    let target_tomorrow = tomorrow
        .and_hms_opt(hour, minute, 0)
        .unwrap_or_else(|| tomorrow.and_hms_opt(20, 0, 0).expect("20:00 is always valid"));
    let delay_tomorrow = (target_tomorrow - now_local).num_seconds().max(1);
    (delay_tomorrow as u64, tomorrow)
}

/// 每日复习提醒调度线程。仅应用运行时生效（窗口关闭后由系统通知中心呈现，
/// 应用完全退出后不再触发）。设定被禁用时每 60 秒探测一次，便于用户重新启用后尽快生效。
pub fn start_reminder_scheduler(
    app: tauri::AppHandle,
    reminder: Arc<Mutex<ReminderSettings>>,
) {
    std::thread::spawn(move || {
        use tauri_plugin_notification::NotificationExt;
        loop {
            let settings = reminder
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_default();
            if !settings.enabled {
                std::thread::sleep(Duration::from_secs(60));
                continue;
            }

            let now = chrono::Local::now();
            let (delay, _date) = next_fire(&now, &settings);
            std::thread::sleep(Duration::from_secs(delay));

            let enabled = reminder
                .lock()
                .map(|guard| guard.enabled)
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let result = app
                .notification()
                .builder()
                .title("Janus · 复习提醒")
                .body("该复习了：打开「今日学习」，完成单词与句子回顾。")
                .show();
            if let Err(error) = result {
                eprintln!("[Janus] reminder notification failed: {error}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn local_at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Local> {
        let naive = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, 0)
            .unwrap();
        Local
            .from_local_datetime(&naive)
            .single()
            .unwrap_or_else(|| Local.from_utc_datetime(&naive))
    }

    fn settings(hour: i64, minute: i64) -> ReminderSettings {
        ReminderSettings {
            enabled: true,
            hour,
            minute,
        }
    }

    #[test]
    fn fires_today_when_time_not_reached() {
        let now = local_at(2026, 11, 20, 9, 30);
        let (delay, date) = next_fire(&now, &settings(20, 0));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 11, 20).unwrap());
        assert_eq!(delay, 10 * 3600 + 30 * 60);
    }

    #[test]
    fn fires_tomorrow_when_time_passed() {
        let now = local_at(2026, 11, 20, 21, 5);
        let (delay, date) = next_fire(&now, &settings(20, 0));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 11, 21).unwrap());
        assert_eq!(delay, 24 * 3600 - (1 * 3600 + 5 * 60));
    }

    #[test]
    fn exact_time_rolls_to_tomorrow() {
        let now = local_at(2026, 11, 20, 20, 0);
        let (delay, date) = next_fire(&now, &settings(20, 0));
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 11, 21).unwrap());
        assert_eq!(delay, 24 * 3600);
    }

    #[test]
    fn clamps_out_of_range_values() {
        let now = local_at(2026, 11, 20, 9, 30);
        let (delay, _date) = next_fire(&now, &settings(99, -5));
        // 99 -> 23, -5 -> 0，即 23:00，距 09:30 为 13.5 小时。
        assert_eq!(delay, 13 * 3600 + 30 * 60);
    }

    #[test]
    fn disabled_settings_still_schedule() {
        let now = local_at(2026, 11, 20, 9, 30);
        let mut disabled = settings(20, 0);
        disabled.enabled = false;
        let (delay, _date) = next_fire(&now, &disabled);
        assert_eq!(delay, 10 * 3600 + 30 * 60);
    }
}
