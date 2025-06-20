use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};

/// Calculates the current epoch number based on a start date (Epoch 0)
pub fn calculate_epoch_express(current_time: DateTime<Utc>) -> i64 {
    // Determine the most recent 7th of the month
    let (epoch_year, epoch_month) = if current_time.day() < 7 {
        if current_time.month() == 1 {
            (current_time.year() - 1, 12)
        } else {
            (current_time.year(), current_time.month() - 1)
        }
    } else {
        (current_time.year(), current_time.month())
    };

    let epoch_start = Utc
        .with_ymd_and_hms(epoch_year, epoch_month, 7, 0, 0, 0)
        .unwrap();

    let duration = current_time.signed_duration_since(epoch_start);
    duration.num_days()
}

/// Gets the next epoch time (midnight UTC of the next day)
pub fn next_epoch_time_express(current_time: DateTime<Utc>) -> DateTime<Utc> {
    let next_day = current_time.date_naive().succ_opt().unwrap();
    Utc.from_utc_datetime(&next_day.and_hms_opt(0, 0, 0).unwrap())
}


/// Epoch 0 starts on Monday, April 10, 2023 (weekly epochs)
const BASE_WEEKLY_EPOCH: &str = "2023-05-01T00:00:00Z";

pub fn calculate_epoch_typical(current_time: DateTime<Utc>) -> i64 {
    let base_time = DateTime::parse_from_rfc3339(BASE_WEEKLY_EPOCH)
        .unwrap()
        .with_timezone(&Utc);
    let duration = current_time.signed_duration_since(base_time);
    duration.num_days() / 7
}

pub fn next_epoch_time_typical(current_time: DateTime<Utc>) -> DateTime<Utc> {
    let base_time = DateTime::parse_from_rfc3339(BASE_WEEKLY_EPOCH)
        .unwrap()
        .with_timezone(&Utc);
    let next_epoch = calculate_epoch_typical(current_time) + 1;
    base_time + Duration::weeks(next_epoch)
}

/// Epoch 0 = Jan 2023
pub fn calculate_epoch_slow(current_time: DateTime<Utc>) -> i64 {
    let base_year = 2023;
    let base_month = 1;

    let year_diff = current_time.year() - base_year;
    let month_diff = current_time.month() as i32 - base_month;

    (year_diff * 12 + month_diff) as i64
}

pub fn next_epoch_time_slow(current_time: DateTime<Utc>) -> DateTime<Utc> {
    let (next_year, next_month) = if current_time.month() == 12 {
        (current_time.year() + 1, 1)
    } else {
        (current_time.year(), current_time.month() + 1)
    };

    Utc.with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_calculate_epoch_express() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 10, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_express(dt), 3);

        let dt = Utc.with_ymd_and_hms(2025, 6, 6, 23, 59, 59).unwrap();
        assert_eq!(calculate_epoch_express(dt), 30);

        let dt = Utc.with_ymd_and_hms(2025, 7, 7, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_express(dt), 0);

        let dt = Utc.with_ymd_and_hms(2025, 6, 19, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_express(dt), 12);
    }

    #[test]
    fn test_next_epoch_time_express() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 17, 12, 0, 0).unwrap();
        let next = next_epoch_time_express(dt);
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 18, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_typical_epoch_values() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 18, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_typical(dt), 111);
        assert_eq!(
            next_epoch_time_typical(dt),
            Utc.with_ymd_and_hms(2025, 6, 23, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_next_epoch_time_typical() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 17, 0, 0, 0).unwrap();
        let next = next_epoch_time_typical(dt);
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 6, 23, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_calculate_epoch_slow() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_slow(dt), 29);

        let dt = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(calculate_epoch_slow(dt), 0);
    }

    #[test]
    fn test_next_epoch_time_slow() {
        let dt = Utc.with_ymd_and_hms(2025, 6, 17, 0, 0, 0).unwrap();
        let next = next_epoch_time_slow(dt);
        assert_eq!(next, Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap());
    }
}

