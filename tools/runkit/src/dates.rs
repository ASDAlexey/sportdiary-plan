use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};

pub const DAY_IDS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

pub fn iso_week_id(date: NaiveDate) -> String {
    let iso = date.iso_week();
    format!("{}-W{:02}", iso.year(), iso.week())
}

pub fn monday_of(date: NaiveDate) -> NaiveDate {
    date - chrono::Duration::days((date.weekday().num_days_from_monday()) as i64)
}

pub fn today_local() -> NaiveDate {
    Local::now().date_naive()
}

pub fn local_date_of_utc(epoch_s: i64) -> NaiveDate {
    let dt: DateTime<Utc> = DateTime::from_timestamp(epoch_s, 0).unwrap_or_else(Utc::now);
    dt.with_timezone(&Local).date_naive()
}

pub fn parse_iso_date(s: &str) -> Result<NaiveDate> {
    let trimmed = s.trim();
    let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .map_err(|e| anyhow!("invalid date {s:?}, expected YYYY-MM-DD: {e}"))?;
    if date.to_string() != trimmed {
        return Err(anyhow!("invalid date {s:?}, expected YYYY-MM-DD"));
    }
    Ok(date)
}

pub fn day_name_ru(date: NaiveDate) -> &'static str {
    const NAMES: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];
    NAMES[date.weekday().num_days_from_monday() as usize]
}

pub fn format_date_ru(date: NaiveDate) -> String {
    format!("{:02}.{:02}", date.day(), date.month())
}

pub fn format_hms(total_s: u32) -> String {
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    let s = total_s % 60;
    format!("{h}:{m:02}:{s:02}")
}

pub fn format_pace(pace_s_per_km: f64) -> String {
    if !pace_s_per_km.is_finite() || pace_s_per_km <= 0.0 || pace_s_per_km >= 3600.0 {
        return "—".to_string();
    }
    let total = pace_s_per_km.round() as u32;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

pub fn format_km(km: f64) -> String {
    let km = if km == 0.0 { 0.0 } else { km };
    let two = (km * 100.0).round() / 100.0;
    let one = (km * 10.0).round() / 10.0;
    if (two - one).abs() < 1e-9 {
        format!("{one:.1}")
    } else {
        format!("{two:.2}")
    }
}

pub fn plural_ru(n: u64, one: &str, few: &str, many: &str) -> String {
    let n10 = n % 10;
    let n100 = n % 100;
    let word = if n10 == 1 && n100 != 11 {
        one
    } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
        few
    } else {
        many
    };
    format!("{n} {word}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_week_id_matches_contract() {
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2026, 10, 5).unwrap()),
            "2026-W41"
        );
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2026, 10, 4).unwrap()),
            "2026-W40"
        );
    }

    #[test]
    fn iso_week_id_handles_year_boundaries() {
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
            "2026-W53"
        );
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2027, 1, 3).unwrap()),
            "2026-W53"
        );
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2027, 1, 4).unwrap()),
            "2027-W01"
        );
        assert_eq!(
            iso_week_id(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            "2026-W01"
        );
    }

    #[test]
    fn monday_of_returns_monday() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
        assert_eq!(monday_of(d), NaiveDate::from_ymd_opt(2026, 9, 28).unwrap());
        let mon = NaiveDate::from_ymd_opt(2026, 10, 5).unwrap();
        assert_eq!(monday_of(mon), mon);
    }

    #[test]
    fn formats_pace_and_duration() {
        assert_eq!(format_hms(3180), "0:53:00");
        assert_eq!(format_hms(1284), "0:21:24");
        assert_eq!(format_hms(3720), "1:02:00");
        assert_eq!(format_pace(305.77), "5:06");
        assert_eq!(format_pace(250.0), "4:10");
        assert_eq!(format_pace(0.0), "—");
    }

    #[test]
    fn formats_km_values() {
        assert_eq!(format_km(42.0), "42.0");
        assert_eq!(format_km(17.5), "17.5");
        assert_eq!(format_km(6.01), "6.01");
        assert_eq!(format_km(23.14159), "23.14");
        assert_eq!(format_km(0.33358), "0.33");
        assert_eq!(format_km(0.0), "0.0");
        assert_eq!(format_km(-0.0), "0.0");
        let empty: Vec<f64> = Vec::new();
        assert_eq!(format_km(empty.iter().sum()), "0.0");
    }

    #[test]
    fn russian_plural_forms() {
        assert_eq!(plural_ru(1, "день", "дня", "дней"), "1 день");
        assert_eq!(plural_ru(2, "день", "дня", "дней"), "2 дня");
        assert_eq!(plural_ru(5, "день", "дня", "дней"), "5 дней");
        assert_eq!(plural_ru(11, "день", "дня", "дней"), "11 дней");
        assert_eq!(plural_ru(21, "день", "дня", "дней"), "21 день");
    }

    #[test]
    fn parses_iso_dates() {
        assert_eq!(
            parse_iso_date("2026-10-05").unwrap(),
            NaiveDate::from_ymd_opt(2026, 10, 5).unwrap()
        );
        assert!(parse_iso_date("2026-10-5").is_err());
        assert!(parse_iso_date("05.10.2026").is_err());
    }
}
