use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct AuthUser {
  pub id: Uuid,
  pub mail: String,

  pub password: String,
}