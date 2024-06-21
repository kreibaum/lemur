use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::cards)]
pub struct NewCard {
    pub place_name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub created_at: NaiveDateTime,
    pub last_reviewed_at: Option<NaiveDateTime>,
    pub next_review_at: NaiveDateTime,
    pub ease_factor: f32,
    pub interval: i32,
}

impl NewCard {
    pub fn new(place_name: String, latitude: f32, longitude: f32) -> Self {
        let now = Utc::now().naive_utc();
        NewCard {
            place_name,
            latitude,
            longitude,
            created_at: now,
            last_reviewed_at: None,
            next_review_at: now,
            ease_factor: 2.5,
            interval: 0,
        }
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::schema::cards)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Card {
    pub id: i32,
    pub place_name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub created_at: NaiveDateTime,
    pub last_reviewed_at: Option<NaiveDateTime>,
    pub next_review_at: NaiveDateTime,
    pub ease_factor: f32,
    pub interval: i32,
}

impl Card {
    pub fn new(place_name: String, latitude: f32, longitude: f32) -> Self {
        let now = Utc::now().naive_utc();
        Card {
            id: 0, // This will be set by the database
            place_name,
            latitude,
            longitude,
            created_at: now,
            last_reviewed_at: None,
            next_review_at: now,
            ease_factor: 2.5,
            interval: 0,
        }
    }
}