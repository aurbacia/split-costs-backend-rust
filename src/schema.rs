// @generated automatically by Diesel CLI.

diesel::table! {
    costs (id) {
        id -> Int4,
        title -> Varchar,
        description -> Text,
        created_at -> Date,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        display_name -> Varchar,
        created_at -> Date,
    }
}

diesel::table! {
    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        token -> Text,
        created_at -> Date,
        expires_at -> Date,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        display_name -> Varchar,
        password -> Varchar,
        created_at -> Date,
    }
}

diesel::table! {
    users_groups (group_id, user_id) {
        group_id -> Int4,
        user_id -> Int4,
        created_at -> Date,
    }
}

diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(users_groups -> groups (group_id));
diesel::joinable!(users_groups -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(costs, groups, sessions, users, users_groups,);
