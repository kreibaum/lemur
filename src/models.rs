use chrono::{Duration, NaiveDateTime, Utc};
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
    const MIN_EASE: f32 = 1.3;
    const MAX_EASE: f32 = 2.5;
    const INITIAL_EASE: f32 = 2.5;
    const INTERVAL_MODIFIER: f32 = 1.0;
    const NEW_INTERVAL: f32 = 0.5;

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

    /// After failing a review, the card is scheduled for review again in 5 minutes, and the ease_factor
    /// and interval are decreased.
    pub fn review_failed(&mut self) {
        let now = chrono::Local::now().naive_local();
        self.last_reviewed_at = Some(now);

        // Decrease ease
        self.ease_factor = (self.ease_factor - 0.2).max(Self::MIN_EASE);

        // Put card into relearning mode
        self.interval = (self.interval as f32 * Self::NEW_INTERVAL).round() as i32;

        // The Card is up for review again in 5 minutes
        self.next_review_at = now + Duration::minutes(5);
    }

    /// After passing a review, update the card's last_reviewed_at, ease_factor, interval, and next_review_at
    /// fields to reflect the current time and the next review time.
    ///
    /// Generally, the interval is increased by the ease_factor, but the interval is also capped at 365 days.
    pub fn review_passed(&mut self) {
        let now = chrono::Local::now().naive_local();
        self.last_reviewed_at = Some(now);

        // Calculate new interval
        self.interval = ((self.interval as f32 * self.ease_factor * Self::INTERVAL_MODIFIER).round() as i32)
            .max(self.interval + 1);
        self.interval = self.interval.min(365);

        // Increase ease
        self.ease_factor = (self.ease_factor + 0.1).min(Self::MAX_EASE);

        self.next_review_at = now + Duration::days(self.interval as i64);
    }

    /// After cramming a card (i.e., reviewing it outside the normal schedule), update the card's
    /// last_reviewed_at and next_review_at fields to reflect the current time and the next review time.
    /// This method does not change the ease_factor or interval fields.
    pub fn review_cramming(&mut self) {
        let now = chrono::Local::now().naive_local();
        self.last_reviewed_at = Some(now);
        self.next_review_at = now + Duration::days(self.interval as i64);
    }
}