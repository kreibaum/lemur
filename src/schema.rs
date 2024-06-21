// @generated automatically by Diesel CLI.

diesel::table! {
    cards (id) {
        id -> Integer,
        place_name -> Text,
        latitude -> Float,
        longitude -> Float,
        created_at -> Timestamp,
        last_reviewed_at -> Nullable<Timestamp>,
        next_review_at -> Timestamp,
        ease_factor -> Float,
        interval -> Integer,
    }
}
