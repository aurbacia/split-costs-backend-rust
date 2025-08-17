// @generated automatically by Diesel CLI.

diesel::table! {
    costs (id) {
        id -> Int4,
        title -> Varchar,
        description -> Text,
        created_at -> Date,
        group_id -> Int4,
        user_id -> Int4,
        amount -> Float8,
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
    payments (id) {
        id -> Int4,
        user_id -> Int4,
        amount -> Float8,
        created_at -> Date,
    }
}

diesel::table! {
    payments_quotas (payment_id, quota_id) {
        payment_id -> Int4,
        quota_id -> Int4,
    }
}

diesel::table! {
    quotas (id) {
        id -> Int4,
        user_id -> Int4,
        cost_id -> Int4,
        percentage_quota -> Float8,
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

diesel::joinable!(costs -> groups (group_id));
diesel::joinable!(costs -> users (user_id));
diesel::joinable!(payments -> users (user_id));
diesel::joinable!(payments_quotas -> payments (payment_id));
diesel::joinable!(payments_quotas -> quotas (quota_id));
diesel::joinable!(quotas -> costs (cost_id));
diesel::joinable!(quotas -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(users_groups -> groups (group_id));
diesel::joinable!(users_groups -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    costs,
    groups,
    payments,
    payments_quotas,
    quotas,
    sessions,
    users,
    users_groups,
);
