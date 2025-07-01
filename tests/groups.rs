#[macro_use]
pub mod common;
#[macro_use]
mod tests {
    use diesel::dsl::exists;
    use diesel::query_dsl::methods::FilterDsl;
    use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl, select};
    use paste::paste;
    use split_costs_rust_backend::database::establish_connection;
    use split_costs_rust_backend::routes::groups::{Payload, ResultBodyGet, ResultBodyPost};
    use split_costs_rust_backend::schema::groups::dsl::*;
    use split_costs_rust_backend::server::RoutesPathes;

    use crate::common::common::{self};

    fn random_payload() -> Payload {
        Payload {
            display_name: fakeit::company::buzzword(),
        }
    }

    #[tokio::test]
    async fn can_create_new_group() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;
        let payload = random_payload();
        let result = server
            .post(RoutesPathes::Groups.as_str())
            .add_cookie(cookie)
            .json(&payload)
            .await
            .json::<ResultBodyPost>();

        let pool = establish_connection();
        let conn = pool.get().await.unwrap();
        let exists = conn
            .interact(move |conn| {
                select(exists(
                    groups.filter(
                        display_name
                            .eq(&payload.display_name)
                            .and(id.eq(&result.id)),
                    ),
                ))
                .get_result::<bool>(conn)
                .unwrap()
            })
            .await
            .unwrap();

        assert!(exists)
    }

    #[tokio::test]
    async fn can_get_all_groups_for_user() {
        let server = common::setup();
        let cookie = common::create_authorized_user_cookies().await;
        let mut groups_ids = Vec::new();
        for _ in 0..3 {
            let payload = random_payload();
            let result = server
                .post(RoutesPathes::Groups.as_str())
                .add_cookie(cookie.clone())
                .json(&payload)
                .await
                .json::<ResultBodyPost>();

            groups_ids.push(result.id);
        }

        let groups_get = server
            .get(RoutesPathes::Groups.as_str())
            .add_cookie(cookie)
            .await
            .json::<ResultBodyGet>();

        assert_eq!(groups_get.groups.len(), 3);
        assert!(
            groups_get
                .groups
                .iter()
                .all(|group| groups_ids.contains(&group.id))
        );
    }

    test_required_json_field!(display_name; RoutesPathes::Groups.as_str(); true);
    test_valid_json_field!(display_name RoutesPathes::Groups.as_str(); fakeit::words::word()[..2].to_string(); true);
}
