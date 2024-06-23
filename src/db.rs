use std::env;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

use crate::models::{Card, NewCard};
use crate::schema::cards;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_card(conn: &mut SqliteConnection, place_name: String, latitude: f32, longitude: f32) -> QueryResult<Card> {
    let new_card = NewCard::new(place_name, latitude, longitude);

    diesel::insert_into(cards::table)
        .values(&new_card)
        .execute(conn)?;

    cards::table.order(cards::id.desc()).first(conn)
}

pub fn get_all_cards(conn: &mut SqliteConnection) -> QueryResult<Vec<Card>> {
    cards::table.load::<Card>(conn)
}

pub fn get_all_due_cards(conn: &mut SqliteConnection) -> QueryResult<Vec<Card>> {
    let now = chrono::Utc::now().naive_utc();
    cards::table
        .filter(cards::next_review_at.le(now))
        .load::<Card>(conn)
}

pub fn count_due_cards(conn: &mut SqliteConnection) -> QueryResult<i64> {
    let now = chrono::Utc::now().naive_utc();
    cards::table
        .filter(cards::next_review_at.le(now))
        .count()
        .get_result(conn)
}

pub fn get_card(conn: &mut SqliteConnection, card_id: i32) -> QueryResult<Card> {
    cards::table.find(card_id).first::<Card>(conn)
}

pub fn update_card(conn: &mut SqliteConnection, updated_card: &Card) -> QueryResult<Card> {
    diesel::update(cards::table.find(updated_card.id))
        .set(updated_card)
        .execute(conn)?;

    cards::table.find(updated_card.id).first(conn)
}

pub fn delete_card(conn: &mut SqliteConnection, card_id: i32) -> QueryResult<usize> {
    diesel::delete(cards::table.find(card_id)).execute(conn)
}

#[cfg(test)]
mod tests {
    use diesel::connection::Connection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use super::*;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    fn establish_test_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .unwrap_or_else(|_| panic!("Error connecting to :memory:"));
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn
    }

    #[test]
    fn test_create_card() {
        let mut conn = establish_test_connection();
        conn.begin_test_transaction().unwrap();

        let card = create_card(&mut conn, "Tokio".to_string(), 35.6895, 139.6917).unwrap();
        assert_eq!(card.place_name, "Tokio");
    }

    #[test]
    fn test_get_all_cards() {
        let mut conn = establish_test_connection();
        conn.begin_test_transaction().unwrap();

        create_card(&mut conn, "London".to_string(), 51.5074, 0.1278).unwrap();
        let cards = get_all_cards(&mut conn).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].place_name, "London");
    }

    #[test]
    fn test_deletion() {
        let mut conn = establish_test_connection();
        conn.begin_test_transaction().unwrap();

        let card = create_card(&mut conn, "Paris".to_string(), 48.8566, 2.3522).unwrap();
        let deleted = delete_card(&mut conn, card.id).unwrap();
        assert_eq!(deleted, 1);
    }
}