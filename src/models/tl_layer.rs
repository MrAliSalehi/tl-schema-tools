use chrono::{NaiveDateTime};

pub struct TlLayer {
    pub layer_id: i32,
    pub layer: String,
    pub release_date: NaiveDateTime,
}