use std::sync::Arc;

use axum::{Form, Json, response::Html, Router, routing::get};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::database_connection_extractor::DatabaseConnection;

mod db;
mod models;
mod schema;
mod database_connection_extractor;
mod gis;

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
        .route("/learn", get(learn_page))
        .route("/api/check_answer", post(check_answer))
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
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(new_card): Form<NewCard>,
) -> Html<String> {
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


async fn all_cards(State(tera): State<Arc<Tera>>,
                   DatabaseConnection(mut conn): DatabaseConnection, ) -> Html<String> {
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
    latitude: f32,
    longitude: f32,
}

async fn learn_page(State(tera): State<Arc<Tera>>,
                    DatabaseConnection(mut conn): DatabaseConnection, ) -> Html<String> {
    use rand::prelude::SliceRandom;

    let cards = db::get_all_due_cards(&mut conn).unwrap_or_else(|_| vec![]);

    if let Some(card) = cards.choose(&mut rand::thread_rng()) {
        let learn_data = LearnPageData {
            id: card.id,
            place_name: card.place_name.clone(),
            latitude: card.latitude,
            longitude: card.longitude,
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


#[derive(Serialize)]
struct AnswerResponse {
    is_correct: bool,
    message: String,
    actual_latitude: f64,
    actual_longitude: f64,
}

async fn check_answer(
    DatabaseConnection(mut conn): DatabaseConnection,
    Form(submission): Form<AnswerSubmission>,
) -> (StatusCode, Json<AnswerResponse>) {
    if let Ok(mut card) = db::get_card(&mut conn, submission.id) {
        let distance = gis::haversine_distance(
            card.latitude, card.longitude,
            submission.latitude, submission.longitude,
        );

        println!("Distance: {:.2}m", distance);

        let (is_correct, message) = if distance <= 250.0 {
            card.review_passed();
            (true, "Correct! Well done!".to_string())
        } else {
            card.review_failed();
            (false, format!("Incorrect. The location was {:.2}m away.", distance))
        };
        let Ok(card) = db::update_card(&mut conn, &card) else {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(AnswerResponse {
                is_correct: false,
                message: "Error updating card".to_string(),
                actual_latitude: 0.0,
                actual_longitude: 0.0,
            }));
        };
        (StatusCode::OK, Json(AnswerResponse {
            is_correct,
            message,
            actual_latitude: card.latitude as f64,
            actual_longitude: card.longitude as f64,
        }))
    } else {
        (StatusCode::NOT_FOUND, Json(AnswerResponse {
            is_correct: false,
            message: "Card not found".to_string(),
            actual_latitude: 0.0,
            actual_longitude: 0.0,
        }))
    }
}
