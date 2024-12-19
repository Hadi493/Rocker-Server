// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        user_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    comments (id) {
        id -> Integer,
        post_id -> Integer,
        user_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(posts -> users (user_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    posts,
    comments,
);
