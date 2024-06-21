use std::sync::Arc;

use axum::{Form, response::Html, Router, routing::get};
use axum::extract::State;
use axum::http::StatusCode;
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
        .route("/learn", get(learn_page).post(check_answer))
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


#[derive(Serialize)]
struct LearnPageData {
    id: i32,
    place_name: String,
}

async fn learn_page(State(tera): State<Arc<Tera>>) -> Html<String> {
    use rand::prelude::SliceRandom;

    let mut conn = db::establish_connection();
    let cards = db::get_all_cards(&mut conn).unwrap_or_else(|_| vec![]);

    if let Some(card) = cards.choose(&mut rand::thread_rng()) {
        let learn_data = LearnPageData {
            id: card.id,
            place_name: card.place_name.clone(),
        };

        let mut context = tera::Context::new();
        context.insert("card", &learn_data);

        let rendered = tera.render("learn.html", &context).unwrap();
        Html(rendered)
    } else {
        Html("<p>No cards available. Please add some cards first.</p>".to_string())
    }
}

#[derive(Deserialize)]
struct AnswerSubmission {
    id: i32,
    latitude: f32,
    longitude: f32,
}

async fn check_answer(Form(submission): Form<AnswerSubmission>) -> (StatusCode, String) {
    let mut conn = db::establish_connection();
    if let Ok(card) = db::get_card(&mut conn, submission.id) {
        let distance = haversine_distance(
            card.latitude, card.longitude,
            submission.latitude, submission.longitude,
        );

        println!("Distance: {:.2}m", distance);

        if distance <= 250.0 {
            (StatusCode::OK, "Correct! Well done!".to_string())
        } else {
            (StatusCode::OK, format!("Incorrect. The location was {:.2}m away.", distance))
        }
    } else {
        (StatusCode::NOT_FOUND, "Card not found".to_string())
    }
}

fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    const R: f32 = 6371.0; // Earth's radius in kilometers

    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    R * c * 1000.0 // Convert to meters
}
