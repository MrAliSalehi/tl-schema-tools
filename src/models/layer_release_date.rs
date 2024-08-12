use chrono::NaiveDate;
use serde::Serialize;

#[derive(Serialize)]
pub struct LayerReleaseDate {
    pub layer_id: i32,
    pub release_date: NaiveDate,
}