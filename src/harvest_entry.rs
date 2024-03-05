use chrono::{IsoWeek, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct TimeEntry {
    pub spent_date: NaiveDate,
    pub hours: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeEntries {
    pub time_entries: Vec<TimeEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct Output {
    pub isoweek: IsoWeek,
    pub expected_hours: f64,
    pub tracked_hours: f64,
    pub diff: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HarvestResp {
    pub time_entries: Vec<TimeEntry>,
    pub next_page: Option<i64>,
}
