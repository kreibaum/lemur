use std::sync::Arc;

use axum::{Form, Json, response::Html, Router, routing::get};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tower_http::compression::CompressionLayer;

use crate::database_connection_extractor::DatabaseConnection;
use crate::models::Card;

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
        .route("/", get(home_page))
        .route("/new_card", get(new_card_form).post(create_new_card))
        .route("/all_cards", get(all_cards))
        .route("/learn", get(select_random_card))
        .route("/learn/:id", get(learn_page))
        .route("/api/check_answer", post(check_answer))
        .layer(CompressionLayer::new())
        .with_state(tera);

    // Run our application
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home_page(
    State(tera): State<Arc<Tera>>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> impl IntoResponse {
    let due_card_count = db::count_due_cards(&mut conn).unwrap_or_else(|_| -1);
    let mut context = Context::new();
    context.insert("due_card_count", &due_card_count);
    let body = tera.render("index.html", &context)
        .unwrap_or_else(|err| format!("Template error: {}", err));

    Html(body).into_response()
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
    next_id: i32,
    /// Should be a 4-character random string. Alpha-numeric characters only.
    cache_buster: String,
}

fn cache_buster() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{Rng, thread_rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

async fn select_random_card(DatabaseConnection(mut conn): DatabaseConnection) -> Result<Redirect, StatusCode> {
    let random_due_card = select_random_card_fkt(&mut conn, -1)?;

    if let Some(card) = random_due_card {
        Ok(Redirect::to(&format!("/learn/{}", card.id)))
    } else {
        Ok(Redirect::to("/"))  // Redirect to home if no cards are available
    }
}

fn select_random_card_fkt(mut conn: &mut SqliteConnection, exclude: i32) -> Result<Option<Card>, StatusCode> {
    use rand::prelude::SliceRandom;

    let cards = db::get_all_due_cards(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter().filter(|card| card.id != exclude).collect::<Vec<_>>();
    let random_due_card = cards.choose(&mut rand::thread_rng());

    Ok(random_due_card.cloned())
}

async fn learn_page(
    State(tera): State<Arc<Tera>>,
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let next_id = match select_random_card_fkt(&mut conn, id) {
        Ok(Some(card)) => card.id,
        _ => -1,
    };

    match db::get_card(&mut conn, id) {
        Ok(card) => {
            let learn_data = LearnPageData {
                id: card.id,
                place_name: card.place_name,
                latitude: card.latitude,
                longitude: card.longitude,
                next_id,
                cache_buster: cache_buster(),
            };

            let mut context = Context::new();
            context.insert("card", &learn_data);

            let rendered = tera.render("learn.html", &context).unwrap();

            // Create a response with appropriate cache control headers
            let mut headers = HeaderMap::new();
            headers.insert("Cache-Control", HeaderValue::from_static("public, max-age=300")); // Cache for 5 minutes

            (headers, Html(rendered)).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Card not found").into_response(),
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
