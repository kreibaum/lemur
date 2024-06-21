use std::sync::Arc;

use axum::{Form, response::Html, Router, routing::get};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use tera::Tera;

mod db;
mod models;
mod schema;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize Tera
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let tera = Arc::new(tera);

    // Build our application with a route
    let app = Router::new()
        .route("/new_card", get(new_card_form).post(create_new_card))
        .route("/all_cards", get(all_cards))
        .with_state(tera);

    // Run our application
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn new_card_form(State(tera): State<Arc<Tera>>) -> Html<String> {
    let context = tera::Context::new();
    let rendered = tera.render("new_card.html", &context).unwrap();
    Html(rendered)
}


#[derive(Deserialize)]
struct NewCard {
    place_name: String,
    latitude: f32,
    longitude: f32,
}

async fn create_new_card(
    State(tera): State<Arc<Tera>>,
    Form(new_card): Form<NewCard>,
) -> Html<String> {
    let mut conn = db::establish_connection();
    match db::create_card(&mut conn, new_card.place_name, new_card.latitude, new_card.longitude) {
        Ok(_) => {
            let mut context = tera::Context::new();
            context.insert("message", "Card created successfully!");
            let rendered = tera.render("new_card.html", &context).unwrap();
            Html(rendered)
        }
        Err(e) => {
            let mut context = tera::Context::new();
            context.insert("error", &format!("Error creating card: {}", e));
            let rendered = tera.render("new_card.html", &context).unwrap();
            Html(rendered)
        }
    }
}


#[derive(Serialize)]
struct CardData {
    id: i32,
    place_name: String,
    latitude: f32,
    longitude: f32,
}


async fn all_cards(State(tera): State<Arc<Tera>>) -> Html<String> {
    let mut conn = db::establish_connection();
    let cards = db::get_all_cards(&mut conn).unwrap_or_else(|_| vec![]);

    let card_data: Vec<CardData> = cards
        .into_iter()
        .map(|card| CardData {
            id: card.id,
            place_name: card.place_name,
            latitude: card.latitude,
            longitude: card.longitude,
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("cards", &card_data);

    let rendered = tera.render("all_cards.html", &context).unwrap();
    Html(rendered)
}