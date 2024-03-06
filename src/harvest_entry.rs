use chrono::{IsoWeek, NaiveDate};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct TimeEntry {
    pub spent_date: NaiveDate,
    pub hours: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeEntries {
    pub time_entries: Vec<TimeEntry>,
}

// Custom serialization function for IsoWeek
fn serialize_iso_week<S>(week: &chrono::naive::IsoWeek, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted_week = format!("{}-W{:02}", week.year(), week.week());
    serializer.serialize_str(&formatted_week)
}

#[derive(Serialize, Debug, Clone)]
pub struct Output {
    #[serde(serialize_with = "serialize_iso_week")]
    pub isoweek: IsoWeek,
    pub month: String,
    pub expected_hours: f64,
    pub tracked_hours: f64,
    pub diff: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HarvestResp {
    pub time_entries: Vec<TimeEntry>,
    pub next_page: Option<i64>,
}
#[derive(Deserialize, Debug)]
pub struct HarvestStatsParams {
    pub harvest_user_id: String,
    pub harvest_token: String,
    pub harvest_account_id: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub expected_hours_per_week: f64,
}

#[derive(Serialize)]
pub struct BeautifulOutput {
    pub output: Output,
    pub accumulated_diff: f64,
}
