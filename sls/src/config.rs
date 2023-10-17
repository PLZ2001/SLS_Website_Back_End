use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::token;

pub const SERVER_URL: [u8; 4] = [127, 0, 0, 1];
pub const SERVER_PORT: u16 = 4000;
pub const FRONT_URL: [u8; 4] = [127, 0, 0, 1];
pub const FRONT_PORT: u16 = 3001;
pub const MONGODB_URL: [u8; 4] = [127, 0, 0, 1];
pub const MONGODB_PORT: u16 = 27017;
pub const DIR_STATIC: &str = "resources/";
pub const DIR_PHOTO_WALL: &str = "images/photo_wall/";
pub const DIR_SLS_MEMBERS: &str = "images/sls_members/";
pub const DIR_FILES: &str = "files/";
pub const API_STATUS_SUCCESS: &str = "SUCCESS";
pub const API_STATUS_FAILURE_WITH_REASONS: &str = "FAILURE_WITH_REASONS";
pub const API_STATUS_FAILURE_WITHOUT_REASONS: &str = "FAILURE_WITHOUT_REASONS";
pub const COOKIES_EXPIRES_SECONDS: f64 = 2000000.0;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct SLSMEMBER {
    pub name: String,
    pub description: String,
    pub image: String,
    pub student_id: String,
    pub introduction: String,
    pub email: String,
    pub phone_number: String,
    pub papers: Vec<Vec<String>>,
    pub paper_years: Vec<String>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct USER {
    pub student_id: String,
    pub name: String,
    pub password: String,
    pub token: token::Token,
    pub sls_verification: bool,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct FILE {
    pub category: String,
    pub name: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct STATS {
    pub watch: i32,
    pub like: i32,
    pub favorite: i32,
    pub comment: i32,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct POST {
    pub post_id: String,
    pub title: String,
    pub content: String,
    pub user_id: String,
    pub time: f64,
    pub stat: STATS,
    pub files: Vec<FILE>,
    pub comment_ids: Vec<String>,
    pub watch_ids: Vec<String>,
    pub like_ids: Vec<String>,
    pub favorite_ids: Vec<String>,
    pub category: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct COMMENT {
    pub comment_id: String,
    pub content: String,
    pub user_id: String,
    pub time: f64,
    pub stat: STATS,
    pub files: Vec<FILE>,
    pub comment_ids: Vec<String>,
    pub watch_ids: Vec<String>,
    pub like_ids: Vec<String>,
    pub favorite_ids: Vec<String>,
}