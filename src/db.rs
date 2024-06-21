

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

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

pub fn get_card(conn: &mut SqliteConnection, card_id: i32) -> QueryResult<Card> {
    cards::table.find(card_id).first::<Card>(conn)
}

pub fn update_card(conn: &mut SqliteConnection, card_id: i32, updated_card: &Card) -> QueryResult<Card> {
    diesel::update(cards::table.find(card_id))
        .set(updated_card)
        .execute(conn)?;

    cards::table.find(card_id).first(conn)
}

pub fn delete_card(conn: &mut SqliteConnection, card_id: i32) -> QueryResult<usize> {
    diesel::delete(cards::table.find(card_id)).execute(conn)
}