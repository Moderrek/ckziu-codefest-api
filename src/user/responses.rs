use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use models::Project;

use crate::posts::api::Post;
use crate::project::models;

#[derive(Serialize)]
pub struct ProfileResponse {
    pub name: String,
    pub display_name: String,
    pub id: Uuid,

    pub bio: Option<String>,

    pub projects: Vec<Project>,
    pub posts: Vec<Post>,

    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    pub updated_at: DateTime<Utc>,

    pub flags: i32,
}

#[derive(Deserialize)]
pub struct PatchUserBody {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
}

#[derive(Deserialize)]
pub struct UpdateBioBody {
    pub bio: String,
}

#[derive(Deserialize)]
pub struct UpdateDisplayNameBody {
    pub displayname: String,
}

#[derive(Serialize)]
pub struct UpdateBioResponse {
    pub success: bool,
    pub message: String,
}
