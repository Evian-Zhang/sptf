use crate::error::{RedisCacheError, SPTFError, SignupError, UnexpectedError, ValidateError};
use deadpool_postgres::Client as PostgresClient;
use deadpool_redis::Connection as RedisConnection;
use log::error;
use sha2::{Digest, Sha256};
use std::future::Future;
use uuid::Uuid;

pub async fn signup_user<
    P1: Future<Output = Result<PostgresClient, Box<dyn SPTFError>>>,
    P2: Future<Output = Result<PostgresClient, Box<dyn SPTFError>>>,
>(
    postgres_client1: P1,
    postgres_client2: P2,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn SPTFError>> {
    let rows = postgres_client1
        .await?
        .query("SELECT id FROM Users WHERE username=$1", &[&username])
        .await
        .map_err(|err| {
            error!("Query username {} failed: {}", username, err);
            UnexpectedError.to_boxed_self()
        })?;
    if !rows.is_empty() {
        return Err(SignupError::UsernameExist.to_boxed_self());
    }
    let uuid = Uuid::new_v4();
    let salt = Uuid::new_v4();
    let salt_bytes = salt.as_bytes();
    let hashed_password = generate_password(&password, &salt_bytes.as_slice());
    postgres_client2
        .await?
        .execute(
            "INSERT INTO Users (id, username, salt, password) VALUES ($1, $2, $3, $4)",
            &[&uuid, &username, &salt_bytes.as_slice(), &hashed_password],
        )
        .await
        .map_err(|err| {
            error!("Failed to create user {}: {}", username, err);
            UnexpectedError.to_boxed_self()
        })?;
    Ok(())
}

fn generate_password(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(&password);
    hasher.finalize().to_vec()
}

/// Validate use given the username and password.
///
/// Return a random-generated UUID as auth-token
pub async fn validate_user<
    P: Future<Output = Result<PostgresClient, Box<dyn SPTFError>>>,
    R: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>,
>(
    postgres_client: P,
    redis_connection: R,
    username: &str,
    password: &str,
) -> Result<Uuid, Box<dyn SPTFError>> {
    let rows = postgres_client
        .await?
        .query(
            "SELECT id, salt, password FROM Users WHERE username=$1",
            &[&username],
        )
        .await
        .map_err(|err| {
            error!("Query username {} failed: {}", username, err);
            UnexpectedError.to_boxed_self()
        })?;
    let row = match &rows[..] {
        [] => {
            return Err(ValidateError::NoUsername.to_boxed_self());
        }
        [row] => row,
        _ => {
            error!("Query username {} returns multiple rows.", username);
            return Err(UnexpectedError.to_boxed_self());
        }
    };
    let id: Uuid = row.try_get(0).map_err(|err| {
        error!("Fetch id field failed: {}", err);
        UnexpectedError.to_boxed_self()
    })?;
    let salt: &[u8] = row.try_get(1).map_err(|err| {
        error!("Fetch salt field failed: {}", err);
        UnexpectedError.to_boxed_self()
    })?;
    let hashed_password: &[u8] = row.try_get(2).map_err(|err| {
        error!("Fetch password field failed: {}", err);
        UnexpectedError.to_boxed_self()
    })?;

    if !validate_password(password, salt, hashed_password) {
        return Err(ValidateError::UnmatchedPassword.to_boxed_self());
    }

    let auth_token = add_user_cache(redis_connection, id).await?;

    Ok(auth_token)
}

fn validate_password(password: &str, salt: &[u8], hashed_password: &[u8]) -> bool {
    &generate_password(password, salt)[..] == hashed_password
}

/// Return randomly generated auth token
async fn add_user_cache<R: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>>(
    connection: R,
    user_uuid: Uuid,
) -> Result<Uuid, Box<dyn SPTFError>> {
    let auth_token = Uuid::new_v4();
    update_user_cache(connection, user_uuid, auth_token).await?;
    Ok(auth_token)
}

async fn update_user_cache<R: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>>(
    connection: R,
    user_uuid: Uuid,
    auth_token: Uuid,
) -> Result<(), Box<dyn SPTFError>> {
    redis::Cmd::set_ex(
        auth_token.to_string(),
        user_uuid.to_string(),
        crate::common::REDIS_CACHE_EXPIRATION_IN_SECONDS,
    )
    .query_async::<_, ()>(&mut connection.await?)
    .await
    .map_err(|err| {
        error!(
            "Update user uuid {} with auth token {} failed: {}",
            user_uuid.to_string(),
            auth_token.to_string(),
            err
        );
        RedisCacheError::UpdateAuthTokenFailed.to_boxed_self()
    })?;
    Ok(())
}

/// Validate user given auth token
///
/// Return user-id
pub async fn validate_auth_token<
    R1: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>,
    R2: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>,
>(
    connection1: R1,
    connection2: R2,
    auth_token_str: &str,
) -> Result<Uuid, Box<dyn SPTFError>> {
    let auth_token = Uuid::parse_str(&auth_token_str).map_err(|err| {
        error!("Parse auth token {} failed: {}", auth_token_str, err);
        RedisCacheError::ValidateAuthTokenFailed.to_boxed_self()
    })?;
    let user_id_string = redis::Cmd::get(auth_token.to_string())
        .query_async::<_, String>(&mut connection1.await?)
        .await
        .map_err(|err| {
            error!(
                "Get user uuid of auth token {} failed: {}",
                auth_token_str, err
            );
            RedisCacheError::ValidateAuthTokenFailed.to_boxed_self()
        })?;
    let user_id = Uuid::parse_str(&user_id_string).map_err(|err| {
        error!("Parse stored user uuid {} failed: {}", user_id_string, err);
        RedisCacheError::ValidateAuthTokenFailed.to_boxed_self()
    })?;
    update_user_cache(connection2, user_id, auth_token).await?;

    Ok(user_id)
}

pub async fn logout<R: Future<Output = Result<RedisConnection, Box<dyn SPTFError>>>>(
    connection: R,
    auth_token_str: &str,
) -> Result<(), Box<dyn SPTFError>> {
    redis::Cmd::del(auth_token_str)
        .query_async::<_, ()>(&mut connection.await?)
        .await
        .map_err(|err| {
            error!("Failed to del auth token {}: {}", auth_token_str, err);
            UnexpectedError.to_boxed_self()
        })?;
    Ok(())
}
