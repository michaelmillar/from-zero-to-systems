#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityRecord {
    #[serde(default)]
    pub check_ins: u32,
    #[serde(default)]
    pub completed: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Progress {
    pub completed: HashSet<String>,
    #[serde(default)]
    pub activity: BTreeMap<String, ActivityRecord>,
}

const FILE: &str = ".play-progress.json";

pub fn load(workspace: &Path) -> Progress {
    let path = workspace.join(FILE);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(workspace: &Path, progress: &Progress) {
    let path = workspace.join(FILE);
    if let Ok(json) = serde_json::to_string_pretty(progress) {
        let _ = std::fs::write(path, json);
    }
}

pub fn record_check_in(workspace: &Path) {
    let today = today_key();
    record_check_in_for_day(workspace, &today);
}

pub fn record_completion(workspace: &Path, package: &str) -> bool {
    let today = today_key();
    record_completion_for_day(workspace, package, &today)
}

pub(crate) fn record_check_in_for_day(workspace: &Path, day: &str) {
    update(workspace, |progress| {
        progress
            .activity
            .entry(day.to_string())
            .or_default()
            .check_ins += 1;
    });
}

pub(crate) fn record_completion_for_day(workspace: &Path, package: &str, day: &str) -> bool {
    let mut inserted = false;
    update(workspace, |progress| {
        if progress.completed.insert(package.to_string()) {
            progress
                .activity
                .entry(day.to_string())
                .or_default()
                .completed += 1;
            inserted = true;
        }
    });
    inserted
}

pub fn streak_days(progress: &Progress) -> u32 {
    streak_days_for(progress, &today_key())
}

pub(crate) fn streak_days_for(progress: &Progress, reference_day: &str) -> u32 {
    let Some(mut day_number) = parse_day_key(reference_day) else {
        return 0;
    };
    let mut streak = 0;

    loop {
        let key = day_key_from_number(day_number);
        let Some(activity) = progress.activity.get(&key) else {
            break;
        };
        if activity.check_ins == 0 && activity.completed == 0 {
            break;
        }

        streak += 1;
        day_number -= 1;
    }

    streak
}

fn update(workspace: &Path, apply: impl FnOnce(&mut Progress)) {
    let mut progress = load(workspace);
    apply(&mut progress);
    save(workspace, &progress);
}

fn today_key() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    day_key_from_number(days)
}

fn parse_day_key(day: &str) -> Option<i64> {
    let mut parts = day.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(days_from_civil(year, month, day))
}

fn day_key_from_number(day_number: i64) -> String {
    let (year, month, day) = civil_from_days(day_number);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let adjusted_year = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = i64::from(month);
    let day_of_year = (153 * (adjusted_month + if adjusted_month > 2 { -3 } else { 9 }) + 2) / 5
        + i64::from(day)
        - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(day_number: i64) -> (i32, u32, u32) {
    let shifted = day_number + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "play-progress-{prefix}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn records_daily_checkins_and_first_time_completions() {
        let root = temp_dir("activity");

        record_check_in_for_day(&root, "2026-03-10");
        record_check_in_for_day(&root, "2026-03-10");
        record_completion_for_day(&root, "alpha", "2026-03-10");
        record_completion_for_day(&root, "alpha", "2026-03-10");

        let progress = load(&root);
        let day = progress
            .activity
            .get("2026-03-10")
            .expect("activity for the day should exist");

        assert_eq!(day.check_ins, 2);
        assert_eq!(day.completed, 1);
        assert!(progress.completed.contains("alpha"));

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[test]
    fn computes_streak_from_consecutive_active_days() {
        let root = temp_dir("streak");

        record_check_in_for_day(&root, "2026-03-10");
        record_check_in_for_day(&root, "2026-03-11");
        record_completion_for_day(&root, "alpha", "2026-03-12");

        let progress = load(&root);
        assert_eq!(streak_days_for(&progress, "2026-03-12"), 3);
        assert_eq!(streak_days_for(&progress, "2026-03-13"), 0);

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }
}
