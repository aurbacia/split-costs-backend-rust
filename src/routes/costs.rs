use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use deadpool_diesel::postgres::Pool;
use diesel::{
    dsl::{exists, insert_into},
    prelude::*,
    select,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    extractors::session::UserSessionClaims,
    models::{Cost, Quota},
    validate::ValidatedPayload,
};

#[derive(Serialize, Deserialize)]
pub struct ResultBody {
    pub expenses: Vec<Cost>,
}

#[derive(Serialize, Deserialize)]
pub struct ResultBodyPost {
    pub added_cost: Cost,
    pub added_quotas: Vec<Quota>,
}

// #[derive(Validate)]
#[derive(Serialize, Deserialize, /* Clone, Default, */ Validate)]
pub struct Payload {
    pub cost: NewCost,
    pub quotas: Vec<NewQuota>,
}

#[derive(Insertable, Validate, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::costs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCost {
    pub group_id: i32,
    pub user_id: i32,

    #[validate(length(min = 3, max = 32))]
    pub title: String,
    pub description: String,

    #[validate(range(min = 0f64))]
    pub amount: f64,
}

#[derive(Insertable, Validate, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::quotas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewQuota {
    pub user_id: i32,
    pub cost_id: Option<i32>,

    #[validate(range(min = 0f64, max = 100f64))]
    pub percentage_quota: f64,
}

pub async fn group_expenses_get(
    user_session: UserSessionClaims,
    State(pool): State<Pool>,
    Path(group_id_from_path): Path<i32>,
) -> impl IntoResponse {
    use crate::schema::{costs, users_groups};

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            let is_member = select(exists(
                users_groups::table
                    .filter(users_groups::user_id.eq(user_session.id))
                    .filter(users_groups::group_id.eq(group_id_from_path)),
            ))
            .get_result::<bool>(conn)?;

            if !is_member {
                return Err(diesel::result::Error::NotFound);
            }

            costs::table
                .filter(costs::group_id.eq(group_id_from_path))
                .select(Cost::as_select())
                .load::<Cost>(conn)
        })
        .await
    {
        Ok(Ok(group_costs)) => Json(ResultBody {
            expenses: group_costs,
        })
        .into_response(),
        Ok(Err(diesel::result::Error::NotFound)) => StatusCode::FORBIDDEN.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[axum::debug_handler]
pub async fn group_expenses_post(
    user_session: UserSessionClaims,
    State(pool): State<Pool>,
    Path(group_id_from_path): Path<i32>,
    ValidatedPayload(payload): ValidatedPayload<Payload>,
) -> impl IntoResponse {
    use crate::schema::{costs, quotas, users_groups};

    let conn = pool.get().await.expect("Can't get connection from pool");
    match conn
        .interact(move |conn| {
            let is_member = select(exists(
                users_groups::table
                    .filter(users_groups::user_id.eq(user_session.id))
                    .filter(users_groups::group_id.eq(group_id_from_path)),
            ))
            .get_result::<bool>(conn)?;

            if !is_member {
                return Err(diesel::result::Error::NotFound);
            }

            let added_cost = insert_into(costs::table)
                .values(payload.cost)
                .returning(Cost::as_returning())
                .get_result::<Cost>(conn)
                .unwrap();

            Ok((
                insert_into(quotas::table)
                    .values::<Vec<NewQuota>>(
                        payload
                            .quotas
                            .iter()
                            .map(|quota| NewQuota {
                                user_id: quota.user_id,
                                cost_id: Some(added_cost.id),
                                percentage_quota: quota.percentage_quota,
                            })
                            .collect(),
                    )
                    .returning(Quota::as_returning())
                    .get_results::<Quota>(conn)
                    .unwrap(),
                added_cost,
            ))
        })
        .await
    {
        Ok(Ok((added_quotas, added_cost))) => Json(ResultBodyPost {
            added_quotas: added_quotas,
            added_cost: added_cost,
        })
        .into_response(),
        Ok(Err(diesel::result::Error::NotFound)) => StatusCode::FORBIDDEN.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
