use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::costs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Cost {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub created_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created_at: time::Date,
    pub expires_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub display_name: String,
    pub created_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::users_groups)]
#[diesel(primary_key(group_id, user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserGroup {
    pub group_id: i32,
    pub user_id: i32,
    pub created_at: time::Date,
}
