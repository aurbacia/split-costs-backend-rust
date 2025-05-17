use jsonwebtoken::*;
use serde::{Serialize, de::DeserializeOwned};

pub fn encode_jwt<'a, T: Serialize + DeserializeOwned>(claims: &T) -> String {
    let secret = std::env::var("JWT_TOKEN_SECRET").expect("JWT_TOKEN_SECRET must be set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn decode_jwt<'a, T: Serialize + DeserializeOwned>(token: &String) -> TokenData<T> {
    let secret = std::env::var("JWT_TOKEN_SECRET").expect("JWT_TOKEN_SECRET must be set");

    decode::<T>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::env;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        exp: usize,
        test_data: String,
    }

    fn random_jwt(seconds: i64) -> TestClaims {
        TestClaims {
            exp: usize::try_from(
                time::UtcDateTime::now().unix_timestamp()
                    + i64::try_from(time::Duration::seconds(seconds).whole_milliseconds()).unwrap(),
            )
            .unwrap(),
            test_data: fakeit::words::sentence(15),
        }
    }

    #[test]
    #[serial_test::serial]
    fn can_encode_and_decode_jwt() {
        dotenvy::dotenv_override().ok();
        let random_claims = random_jwt(60);
        let token = encode_jwt(&random_claims);
        let decoded_claims = decode_jwt::<TestClaims>(&token);
        assert_eq!(decoded_claims.claims.test_data, random_claims.test_data);
        assert_eq!(decoded_claims.claims.exp, random_claims.exp);
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "JWT_TOKEN_SECRET must be set")]
    fn cant_encode_jwt_without_env_secret() {
        dotenvy::dotenv_override().ok();
        unsafe {
            env::remove_var("JWT_TOKEN_SECRET");
        }

        encode_jwt(&random_jwt(60));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "JWT_TOKEN_SECRET must be set")]
    fn cant_decode_jwt_without_env_secret() {
        dotenvy::dotenv_override().ok();
        let token = encode_jwt(&random_jwt(60));
        unsafe {
            env::remove_var("JWT_TOKEN_SECRET");
        }

        decode_jwt::<TestClaims>(&token);
    }

    #[test]
    #[serial_test::serial]
    #[should_panic]
    fn cant_decode_jwt_if_expired() {
        dotenvy::dotenv_override().ok();
        let token = encode_jwt(&random_jwt(1));
        std::thread::sleep(std::time::Duration::from_secs(2));

        unsafe {
            env::remove_var("JWT_TOKEN_SECRET");
        }

        decode_jwt::<TestClaims>(&token);
    }
}
