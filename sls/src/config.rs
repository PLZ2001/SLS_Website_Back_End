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
pub const API_STATUS_SUCCESS: &str = "SUCCESS";
pub const API_STATUS_FAILURE_WITH_REASONS: &str = "FAILURE_WITH_REASONS";
pub const API_STATUS_FAILURE_WITHOUT_REASONS: &str = "FAILURE_WITHOUT_REASONS";
pub const COOKIES_EXPIRES_SECONDS: f64 = 2000000.0;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct USER {
    pub student_id: String,
    pub name: String,
    pub grade: String,
    pub password: String,
    pub token: token::Token,
}