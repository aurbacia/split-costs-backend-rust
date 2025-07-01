use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use deadpool_diesel::postgres::Pool;
use diesel::{dsl::insert_into, prelude::*};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{extractors::session::UserSessionClaims, models::Group, validate::ValidatedPayload};

pub async fn groups_post(
    user_session: UserSessionClaims,
    State(pool): State<Pool>,
    ValidatedPayload(payload): ValidatedPayload<Payload>,
) -> impl IntoResponse {
    use crate::schema::groups::dsl::*;
    use crate::schema::users_groups::dsl::*;

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            let added_group = insert_into(groups)
                .values(payload)
                .get_result::<Group>(conn)
                .unwrap();

            insert_into(users_groups)
                .values((group_id.eq(added_group.id), user_id.eq(user_session.id)))
                .execute(conn)
                .unwrap();

            added_group
        })
        .await
    {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(group) => Json(ResultBodyPost { id: group.id }).into_response(),
    }
}

pub async fn groups_get(
    user_session: UserSessionClaims,
    State(pool): State<Pool>,
) -> impl IntoResponse {
    use crate::schema::groups::dsl::*;
    use crate::schema::users_groups::dsl::*;

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            let groups_ids = users_groups
                .select(group_id)
                .filter(user_id.eq(user_session.id))
                .load::<i32>(conn)
                .unwrap();

            groups
                .select(Group::as_select())
                .filter(id.eq_any(groups_ids))
                .load::<Group>(conn)
                .unwrap()
        })
        .await
    {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(selected_groups) => Json(ResultBodyGet {
            groups: selected_groups,
        })
        .into_response(),
    }
}

#[derive(Serialize, Deserialize, Insertable, Clone, Validate)]
#[diesel(table_name = crate::schema::groups)]
pub struct Payload {
    #[validate(length(min = 3, max = 32))]
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResultBodyPost {
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ResultBodyGet {
    pub groups: Vec<Group>,
}
