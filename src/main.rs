use crate::db::{create_card, establish_connection};

mod db;
mod models;
mod schema;

fn main() {
    let mut connection = establish_connection();
    let card = create_card(&mut connection, "Tokyo".to_string(), 35.6895, 139.6917).expect("Error creating card");
    let cards = db::get_all_cards(&mut connection).expect("Error getting cards");
    for card in cards {
        println!("{:?}", card);
    }
    db::delete_card(&mut connection, card.id).expect("Error deleting card");
}
