use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::users::CodeFestUser;



#[derive(FromRow, Serialize, Deserialize)]
struct Project {
    id: Uuid,
    name: String,
    display_name: String,
    
    owner_id: Uuid,
    
    private: bool,
    description: Option<String>,
    
    likes: u32,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct GetProjectResponse {
    id: Uuid,
    name: String,
    display_name: String,
    
    owner: CodeFestUser,
    
    private: bool,
    description: Option<String>,
    
    likes: u32,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}