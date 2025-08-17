#[macro_use]
pub mod common;

#[macro_use]
mod tests {
    use std::sync::Arc;

    use axum_extra::extract::cookie::Cookie;
    use axum_test::TestServer;
    use deadpool_diesel::postgres::Pool;
    use diesel::{
        dsl::{exists, select},
        prelude::*,
    };
    use split_costs_rust_backend::{
        database::establish_connection,
        extractors::session::decode_jwt_user_session,
        routes::{
            costs::{
                NewCost, NewQuota, Payload as CostPayload, ResultBody, ResultBodyPost as CostResult,
            },
            groups::{Payload as GroupPayload, ResultBodyPost as GroupResultBodyPost},
        },
        schema::{costs, quotas, users_groups},
        server::RoutesPathes,
    };
    use tokio_stream::StreamExt;

    use crate::common::common;

    fn random_group_payload() -> GroupPayload {
        GroupPayload {
            display_name: fakeit::company::buzzword(),
        }
    }

    async fn create_group(server: &TestServer, cookie: &Cookie<'_>) -> i32 {
        let payload = random_group_payload();
        server
            .post(RoutesPathes::Groups.as_str())
            .add_cookie(cookie.clone())
            .json(&payload)
            .await
            .json::<GroupResultBodyPost>()
            .id
    }

    async fn create_test_cost(
        pool: &Pool,
        server: &TestServer,
        cookie: &Cookie<'_>,
        g_id: i32,
        u_id: i32,
    ) -> CostResult {
        let conn = pool.get().await.unwrap();
        let mut quotas = Vec::<NewQuota>::new();
        let mut quota_perc = 0;
        while quota_perc < 100 {
            // NOTE: very slow :(
            let cookie = common::create_authorized_user_cookies().await;
            let session_token = cookie.value().to_string();
            let claims = decode_jwt_user_session(&session_token);
            let new_user_id = claims.claims.id;

            let mut quota = fakeit::misc::random(0, 101 - quota_perc);
            if (quota_perc > 70) {
                quota = 100 - quota_perc;
            }

            quota_perc += quota;
            quotas.push(NewQuota {
                user_id: new_user_id,
                cost_id: None,
                percentage_quota: quota.into(),
            });
        }

        let arc_quotas = Arc::new(quotas.clone());
        conn.interact(move |conn| {
            let cloned_quotas = arc_quotas.clone();
            diesel::insert_into(users_groups::table)
                .values(Vec::from_iter(cloned_quotas.iter().map(|q| {
                    return (
                        users_groups::group_id.eq(g_id),
                        users_groups::user_id.eq(q.user_id),
                    );
                })))
                .execute(conn)
        })
        .await
        .unwrap()
        .unwrap();

        server
            .post(
                &RoutesPathes::GroupExpenses
                    .as_str()
                    .replace("{id}", &g_id.to_string()),
            )
            .add_cookie(cookie.clone())
            .json(&CostPayload {
                cost: NewCost {
                    group_id: g_id,
                    user_id: u_id,
                    title: fakeit::words::sentence(3),
                    description: fakeit::words::sentence(5),
                    amount: fakeit::currency::price(50f64, 1500f64),
                },
                quotas: quotas.clone().to_vec(),
            })
            .await
            .json::<CostResult>()
    }

    #[tokio::test]
    async fn can_get_group_expenses_as_member() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;

        let group_id = create_group(&server, &cookie).await;

        let session_token = cookie.value().to_string();
        let claims = decode_jwt_user_session(&session_token);
        let user_id = claims.claims.id;

        let pool = establish_connection();
        let mut created_costs_ids = Vec::new();
        for _ in 0..3 {
            let cost = create_test_cost(&pool, &server, &cookie, group_id, user_id).await;
            created_costs_ids.push(cost.added_cost.id);
        }

        let response = server
            .get(&format!("/groups/{}/expenses", group_id))
            .add_cookie(cookie)
            .await;

        response.assert_status_ok();
        let body = response.json::<ResultBody>();
        assert_eq!(body.expenses.len(), 3);
        assert!(
            body.expenses
                .iter()
                .all(|cost| created_costs_ids.contains(&cost.id))
        );
    }

    #[tokio::test]
    async fn get_empty_list_for_group_with_no_expenses() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;

        let group_id = create_group(&server, &cookie).await;

        let response = server
            .get(&format!("/groups/{}/expenses", group_id))
            .add_cookie(cookie)
            .await;

        response.assert_status_ok();
        let body = response.json::<ResultBody>();
        assert!(body.expenses.is_empty());
    }

    #[tokio::test]
    async fn cant_get_group_expenses_as_non_member() {
        let server = common::setup();
        let cookie1 = common::create_authorized_user_cookies().await;

        let group_id = create_group(&server, &cookie1).await;

        let cookie2 = common::create_authorized_user_cookies().await;
        server
            .get(&format!("/groups/{}/expenses", group_id))
            .add_cookie(cookie2)
            .await
            .assert_status_forbidden();
    }

    #[tokio::test]
    async fn get_forbidden_for_non_existent_group() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;
        let non_existent_group_id = 99999;

        server
            .get(&format!("/groups/{}/expenses", non_existent_group_id))
            .add_cookie(cookie)
            .await
            .assert_status_forbidden();
    }

    #[tokio::test]
    async fn can_add_expense_to_group() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;

        let group_id = create_group(&server, &cookie).await;

        let session_token = cookie.value().to_string();
        let claims = decode_jwt_user_session(&session_token);
        let user_id = claims.claims.id;

        let pool = establish_connection();
        let cost = create_test_cost(&pool, &server, &cookie, group_id, user_id).await;

        let conn = pool.get().await.unwrap();
        let exists_cost = conn
            .interact(move |conn| {
                select(exists(
                    costs::table.filter(
                        costs::id.eq(cost.added_cost.id).and(
                            costs::amount.eq(cost.added_cost.amount).and(
                                costs::title
                                    .eq(cost.added_cost.title)
                                    .and(costs::user_id.eq(user_id)),
                            ),
                        ),
                    ),
                ))
                .get_result::<bool>(conn)
                .unwrap()
            })
            .await
            .unwrap();

        tokio::task::spawn(async move {
            let mut stream = tokio_stream::iter(cost.added_quotas);
            while let Some(quota) = stream.next().await {
                let exists = conn
                    .interact(move |conn| {
                        select(exists(
                            quotas::table.filter(
                                quotas::cost_id.eq(quota.cost_id).and(
                                    quotas::percentage_quota
                                        .eq(quota.percentage_quota)
                                        .and(quotas::id.eq(quota.id)),
                                ),
                            ),
                        ))
                        .get_result::<bool>(conn)
                        .unwrap()
                    })
                    .await
                    .unwrap();

                assert!(exists)
            }
        });

        assert!(exists_cost)
    }
}
