use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::costs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Cost {
    pub id: i32,
    pub group_id: i32,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub amount: f64, // NOTE: can be f16 and f32 as well, but f64 and f32 has roughly the same speed, and f16 is not supported on many platforms
    pub created_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::quotas)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Cost))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Quota {
    pub id: i32,
    pub user_id: i32,
    pub cost_id: i32,
    pub percentage_quota: f64,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Payment {
    pub id: i32,
    pub user_id: i32,
    pub amount: f64,
    pub created_at: time::Date,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::payments_quotas)]
#[diesel(belongs_to(Quota))]
#[diesel(belongs_to(Payment))]
#[diesel(primary_key(quota_id, payment_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentQuota {
    pub quota_id: i32,
    pub payment_id: i32,
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

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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
