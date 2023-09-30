use serde::{Deserialize, Serialize};

pub const SERVER_URL: [u8; 4] = [127, 0, 0, 1];
pub const SERVER_PORT: u16 = 4000;
pub const MONGODB_URL: [u8; 4] = [127, 0, 0, 1];
pub const MONGODB_PORT: u16 = 27017;
pub const DIR_STATIC: &str = "resources/";
pub const DIR_PHOTO_WALL: &str = "images/photo_wall/";
pub const DIR_SLS_MEMBERS: &str = "images/sls_members/";

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct USER {
    pub name: String,
    pub grade: String,
    pub password: String,
}