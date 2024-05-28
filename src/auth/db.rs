use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::models::AuthUser;
use crate::user::models::User;

const IS_USER_EXISTS_QUERY: &str = r"SELECT auth.id FROM auth INNER JOIN users ON auth.id = users.id WHERE users.name = $1 or auth.mail = $2 LIMIT 1";

pub async fn is_user_exists(name: &String, mail: &String, pool: &PgPool) -> Result<bool, Box<dyn std::error::Error>> {
  let optional = sqlx::query(IS_USER_EXISTS_QUERY)
    .bind(name)
    .bind(mail)
    .fetch_optional(pool)
    .await?;

  Ok(optional.is_some())
}

const GET_USER_PASSWORD_UUID_QUERY: &str = r"SELECT auth.password, auth.id, users.name FROM auth INNER JOIN users ON auth.id = users.id WHERE users.name = $1 OR auth.mail = $1 LIMIT 1";

pub async fn get_user_password_uuid(login: &String, pool: &PgPool) -> Result<Option<(String, Uuid, String, )>, Box<dyn std::error::Error>> {
  let result: Option<(String, Uuid, String, )> = sqlx::query_as(GET_USER_PASSWORD_UUID_QUERY)
    .bind(login)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}

const REGISTER_USER_AUTH_QUERY: &str = r"INSERT INTO auth (mail, id, password) VALUES ($1, $2, $3)";
const REGISTER_USER_QUERY: &str = r"INSERT INTO users (name, display_name, id, bio, created_at, updated_at, flags) VALUES ($1, $2, $3, $4, $5, $6, $7)";

pub async fn register_user(auth_user: &AuthUser, user: &User, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
  let mut transaction = pool.begin().await?;

  sqlx::query(REGISTER_USER_AUTH_QUERY)
    .bind(&auth_user.mail)
    .bind(auth_user.id)
    .bind(&auth_user.password)
    .execute(&mut *transaction)
    .await?;

  sqlx::query(REGISTER_USER_QUERY)
    .bind(&user.name)
    .bind(&user.display_name)
    .bind(user.id)
    .bind(&user.bio)
    .bind(user.created_at)
    .bind(user.updated_at)
    .bind(user.flags)
    .execute(&mut *transaction)
    .await?;

  transaction.commit().await?;

  Ok(())
}
