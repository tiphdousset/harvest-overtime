mod harvest_entry;
use crate::harvest_entry::*;
use chrono::{Datelike, IsoWeek, Month, NaiveDate};
use colored::*;
use group_by::group_by;
use num_traits::cast::FromPrimitive;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let from_env = env::var("FROM").expect("$FROM is not set");
    let from: NaiveDate = NaiveDate::parse_from_str(&from_env, "%Y-%m-%d").unwrap();
    let to_env = env::var("TO").expect("$TO is not set");
    let to: NaiveDate = NaiveDate::parse_from_str(&to_env, "%Y-%m-%d").unwrap();
    let expected_hours_per_week: f64 = env::var("WEEKLY_HOURS")
        .expect("$WEEKLY_HOURS is not set")
        .parse()
        .unwrap();
    let expected_isoweeks: Vec<IsoWeek> = from
        .iter_weeks()
        .take_while(|w| w.iso_week() <= to.iso_week())
        .map(|w| w.iso_week())
        .collect();

    let user_id = env::var("HARVEST_USER_ID").expect("$HARVEST_USER_ID is not set");
    let token = env::var("HARVEST_ACCESS_TOKEN").expect("$HARVEST_ACCESS_TOKEN is not set");
    let account_id = env::var("HARVEST_ACCOUNT_ID").expect("$HARVEST_ACCOUNT_ID is not set");

    println!(
        "Getting time entries for user_id: {user_id} for timerange from: {from_env} to: {to_env}. \n Expected hours per week: {expected_hours_per_week} hours."
    );
    let time_entries: Vec<TimeEntry> =
        get_time_entries(user_id, from_env, to_env, account_id, token).await?;
    let time_entries_filtered: TimeEntries = TimeEntries {
        time_entries: time_entries
            .iter()
            .filter(|te| te.spent_date >= from && te.spent_date <= to)
            .map(|te| *te)
            .collect(),
    };

    let time_entries_per_isoweek = group_by(time_entries_filtered.time_entries, |te| {
        te.spent_date.iso_week()
    });

    let empty_vec: Vec<TimeEntry> = vec![];
    let time_entries_with_details: Vec<Output> = expected_isoweeks
        .iter()
        .map(|&week| {
            let entries = time_entries_per_isoweek.get(&week).unwrap_or(&empty_vec);
            let expected_hours_this_week =
                due_hours_per_week(week, from, to, expected_hours_per_week);
            let tracked_hours_this_week = entries.iter().map(|te| te.hours).sum::<f64>();
            Output {
                isoweek: week,
                tracked_hours: tracked_hours_this_week,
                expected_hours: expected_hours_this_week,
                diff: tracked_hours_this_week - expected_hours_this_week,
            }
        })
        .collect();

    let output_sorted_with_accumulated_overtime: Vec<(Output, f64)> = time_entries_with_details
        .iter()
        .scan(0.0, |acc, &output| {
            *acc += output.diff;
            Some((output, *acc))
        })
        .collect();

    output_sorted_with_accumulated_overtime
        .iter()
        .for_each(|(output, acc)| {
            let output_formatted = display_output(*output, *acc);
            println!("{}", output_formatted);
        });
    Ok(())
}

async fn get_time_entries(
    user_id: String,
    from: String,
    to: String,
    account_id: String,
    token: String,
) -> Result<Vec<TimeEntry>, Box<dyn std::error::Error>> {
    let mut time_entries: Vec<TimeEntry> = vec![];
    let params: [(&str, &str); 3] = [
        ("user_id", user_id.as_str()),
        ("from", from.as_str()),
        ("to", to.as_str()),
    ];
    let mut next_page = 1;
    loop {
        println!("Getting page {next_page} of time entries.");
        let mut foo = get(params, account_id.clone(), token.clone(), next_page).await?;
        time_entries.append(&mut foo.time_entries);
        match foo.next_page {
            Some(page) => next_page = page,
            None => break,
        }
    }
    Ok(time_entries)
}

async fn get(
    params: [(&str, &str); 3],
    account_id: String,
    token: String,
    page: i64,
) -> Result<HarvestResp, Box<dyn std::error::Error>> {
    let param_page = [("page", page)];
    let client = reqwest::Client::new();
    let harvest_resp = client
        .get("https://api.harvestapp.com/api/v2/time_entries")
        .query(&params)
        .query(&param_page)
        .header("Harvest-Account-ID", account_id)
        .header("Authorization", "Bearer ".to_string() + token.as_str())
        .header("User-Agent", "Harvest API Example")
        .send()
        .await?
        .json::<HarvestResp>()
        .await?;
    Ok(harvest_resp)
}

fn due_hours_per_week(
    isoweek: IsoWeek,
    from: NaiveDate,
    to: NaiveDate,
    expected_hours_per_week: f64,
) -> f64 {
    println!(
        "Calculating expected hours for week: {:?}, \n From: {:?} ({:?}), \n To: {:?} ({:?}), \n Expected_hours_per_week: {:?} \n",
        isoweek, from, from.iso_week(), to, to.iso_week(), expected_hours_per_week
    );
    let expected_hours_per_day = expected_hours_per_week / 5.0;
    let start_weekday = from.weekday().number_from_monday();
    let start_in_weekend = start_weekday > 5;
    let start_isoweek = from.iso_week();
    let end_weekday = to.weekday().number_from_monday();
    let end_in_weekend = end_weekday > 5;
    let end_isoweek = to.iso_week();
    let outside = isoweek < start_isoweek || isoweek > end_isoweek;

    if outside {
        println!(
            "Given week: {:?} is outside of give time range: from: {:?} to: {:?}. No expected working hours.",
            isoweek, from, to
        );
        return 0.0;
    } else if start_in_weekend && isoweek == start_isoweek {
        println!("Only weekend days in the current week. No expected working hours.");
        return 0.0;
    } else if start_isoweek == isoweek && end_isoweek == isoweek {
        println!("Given time range starts and ends the same week.");
        let days_worked_in_week = ((end_weekday - start_weekday) + 1) as f64;
        let hours_worked_in_week = days_worked_in_week * expected_hours_per_day;
        return hours_worked_in_week;
    } else if start_isoweek == isoweek {
        println!("Currently caluculating expected hours for the start week.");
        let days_worked_in_start_week = (5 - start_weekday + 1) as f64;
        let hours_worked_in_start_week = days_worked_in_start_week * expected_hours_per_day;
        return hours_worked_in_start_week;
    } else if end_isoweek == isoweek {
        println!("Currently caluculating expected hours for the last week.");
        if end_in_weekend {
            println!("Full last week");
            return expected_hours_per_week;
        } else {
            return end_weekday as f64 * expected_hours_per_day;
        };
    } else {
        println!("Currently calculating expected hours for a week not being start/end week.");
        return expected_hours_per_week;
    }
}

fn display_output(output: Output, accumulated_overtime: f64) -> String {
    let diff_of_the_week = output.diff;
    let hashes_expected = "#".repeat(output.expected_hours as u32 as usize);
    let hashes_tracked = "#".repeat(output.tracked_hours as u32 as usize);
    let isoweek = output.isoweek;
    let month = NaiveDate::from_isoywd_opt(isoweek.year(), isoweek.week(), chrono::Weekday::Mon)
        .unwrap()
        .month();
    let month_name: String = Month::from_u32(month).unwrap().name().to_string();
    let tracked_hours = output.tracked_hours;
    let hashes: String = if diff_of_the_week < 0.0 {
        format!(
            "{}{}",
            hashes_tracked.yellow(),
            "#".repeat(diff_of_the_week.abs() as u32 as usize).red()
        )
    } else {
        format!(
            "{}{}",
            hashes_expected.yellow(),
            "#".repeat(diff_of_the_week as u32 as usize).green()
        )
    };
    return format_output(
        isoweek,
        month_name,
        tracked_hours,
        hashes,
        diff_of_the_week,
        accumulated_overtime,
    );
}

fn format_output(
    isoweek: IsoWeek,
    month_name: String,
    tracked_hours: f64,
    hashes: String,
    diff_of_the_week: f64,
    accumulated_overtime: f64,
) -> String {
    return format!(
        "{:?} {:10} {:6.2}h (tracked) {:60} {:+6.2}h (this week) {acc:+6.2}h (accumulated)",
        isoweek,
        month_name,
        tracked_hours,
        hashes,
        diff_of_the_week,
        acc = accumulated_overtime,
    );
}

#[test]
fn test_due_hours_per_week_start_monday_end_friday_same_week() {
    let expected_hours_per_week = 30.0;
    let from_monday = NaiveDate::parse_from_str("2024-01-22", "%Y-%m-%d").unwrap();
    let to_friday: NaiveDate = NaiveDate::parse_from_str("2024-01-26", "%Y-%m-%d").unwrap();
    let isoweek_from = from_monday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_monday,
        to_friday,
        expected_hours_per_week,
    );
    let isoweek_one_month_after_end_time = NaiveDate::parse_from_str("2024-02-26", "%Y-%m-%d")
        .unwrap()
        .iso_week();
    let due_hours_outside_range_after = due_hours_per_week(
        isoweek_one_month_after_end_time,
        from_monday,
        to_friday,
        expected_hours_per_week,
    );
    let isoweek_one_month_before_start_time = NaiveDate::parse_from_str("2023-12-22", "%Y-%m-%d")
        .unwrap()
        .iso_week();
    let due_hours_outside_range_before = due_hours_per_week(
        isoweek_one_month_before_start_time,
        from_monday,
        to_friday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 30.0);
    assert_eq!(due_hours_outside_range_after, 00.0);
    assert_eq!(due_hours_outside_range_before, 00.0);
}

#[test]
fn test_due_hours_per_week_start_tuesday_end_thursday_same_week() {
    let expected_hours_per_week = 30.0;
    let from_tuesday = NaiveDate::parse_from_str("2024-01-23", "%Y-%m-%d").unwrap();
    let to_thursday: NaiveDate = NaiveDate::parse_from_str("2024-01-25", "%Y-%m-%d").unwrap();
    let isoweek_from = from_tuesday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_tuesday,
        to_thursday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 18.0);
}

#[test]
fn test_due_hours_per_week_start_tuesday_end_thursday_1_week_apart() {
    let expected_hours_per_week = 30.0;
    let from_tuesday = NaiveDate::parse_from_str("2024-01-16", "%Y-%m-%d").unwrap();
    let to_thursday: NaiveDate = NaiveDate::parse_from_str("2024-01-25", "%Y-%m-%d").unwrap();
    let isoweek_from = from_tuesday.iso_week();
    let isoweek_end = to_thursday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_tuesday,
        to_thursday,
        expected_hours_per_week,
    );
    let due_hours_last_week = due_hours_per_week(
        isoweek_end,
        from_tuesday,
        to_thursday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 24.0);
    assert_eq!(due_hours_last_week, 24.0);
}

#[test]
fn test_start_saturday_end_tuesday_the_week_after() {
    let expected_hours_per_week = 30.0;
    let from_saturday: NaiveDate = NaiveDate::parse_from_str("2024-01-27", "%Y-%m-%d").unwrap();
    let to_tuesday = NaiveDate::parse_from_str("2024-01-30", "%Y-%m-%d").unwrap();
    let isoweek_from = from_saturday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_saturday,
        to_tuesday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 0.0);
}

#[test]
fn test_start_sunday_end_tuesday_the_week_after() {
    let expected_hours_per_week = 30.0;
    let from_sunday: NaiveDate = NaiveDate::parse_from_str("2024-01-28", "%Y-%m-%d").unwrap();
    let to_tuesday = NaiveDate::parse_from_str("2024-01-30", "%Y-%m-%d").unwrap();
    let isoweek_from = from_sunday.iso_week();
    let isoweek_end = to_tuesday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_sunday,
        to_tuesday,
        expected_hours_per_week,
    );
    let due_hours_last_week = due_hours_per_week(
        isoweek_end,
        from_sunday,
        to_tuesday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 0.0);
    assert_eq!(due_hours_last_week, 12.0);
}

#[test]
fn test_start_saturday_end_sunday_same_week() {
    let expected_hours_per_week = 30.0;
    let from_saturday: NaiveDate = NaiveDate::parse_from_str("2024-01-27", "%Y-%m-%d").unwrap();
    let to_sunday = NaiveDate::parse_from_str("2024-01-28", "%Y-%m-%d").unwrap();
    let isoweek_from = from_saturday.iso_week();
    let isoweek_end = to_sunday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_saturday,
        to_sunday,
        expected_hours_per_week,
    );
    let due_hours_last_week = due_hours_per_week(
        isoweek_end,
        from_saturday,
        to_sunday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 0.0);
    assert_eq!(due_hours_last_week, 0.0);
}

#[test]
fn test_start_saturday_end_sunday_following_week() {
    let expected_hours_per_week = 30.0;
    let from_saturday: NaiveDate = NaiveDate::parse_from_str("2024-02-17", "%Y-%m-%d").unwrap();
    let to_sunday = NaiveDate::parse_from_str("2024-02-25", "%Y-%m-%d").unwrap();
    let isoweek_from = from_saturday.iso_week();
    let isoweek_end = to_sunday.iso_week();
    let due_hours_start_week = due_hours_per_week(
        isoweek_from,
        from_saturday,
        to_sunday,
        expected_hours_per_week,
    );
    let due_hours_last_week = due_hours_per_week(
        isoweek_end,
        from_saturday,
        to_sunday,
        expected_hours_per_week,
    );

    assert_eq!(due_hours_start_week, 0.0);
    assert_eq!(due_hours_last_week, 30.0);
}
