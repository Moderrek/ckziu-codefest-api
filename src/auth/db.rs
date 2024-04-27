use sqlx::PgPool;
use uuid::Uuid;

pub async fn is_user_exists(login: &String, pool: &PgPool) -> Result<bool, Box<dyn std::error::Error>> {
  let query = "SELECT auth.id FROM auth INNER JOIN users ON auth.id = users.id WHERE users.name = $1 or auth.mail = $1 LIMIT 1";

  let optional = sqlx::query(query)
    .bind(login)
    .fetch_optional(pool)
    .await?;

  Ok(optional.is_some())
}

pub async fn get_user_password_uuid(login: &String, pool: &PgPool) -> Result<Option<(String, Uuid, String, )>, Box<dyn std::error::Error>> {
  let query = "SELECT auth.password, auth.id, users.name FROM auth INNER JOIN users ON auth.id = users.id WHERE users.name = $1 OR auth.mail = $1 LIMIT 1";

  let result: Option<(String, Uuid, String, )> = sqlx::query_as(query)
    .bind(login)
    .fetch_optional(pool)
    .await?;

  Ok(result)
}