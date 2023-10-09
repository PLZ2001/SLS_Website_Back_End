use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use mongodb::bson::doc;
use mongodb::Client;
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::{config, token};

#[derive(Debug)]
pub struct FailedToUpdateToken(Box<String>);

impl warp::reject::Reject for FailedToUpdateToken {}

#[derive(Debug)]
pub struct FailedToValidateToken(Box<String>);

impl warp::reject::Reject for FailedToValidateToken {}

#[derive(Debug)]
pub struct FailedToClearToken(Box<String>);

impl warp::reject::Reject for FailedToClearToken {}


#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Token {
    pub token: String,
    pub expires: f64,
    pub student_id_of_token: String,
}

impl Token {
    pub fn new(student_id: String) -> Result<Self, String> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => {
                let token = digest(format!("{}+{}", student_id, time.as_secs().to_string()));
                let expires = time.as_secs_f64() + config::COOKIES_EXPIRES_SECONDS;
                return Ok(Token { token, expires, student_id_of_token: student_id });
            }
            Err(_) => {
                return Err(String::from("服务器时间存在致命错误"));
            }
        }
    }
    pub fn new_empty() -> Self {
        return Token { token: String::from(""), expires: 0.0, student_id_of_token: String::from("") };
    }
}

pub async fn validate_token(token: &Token) -> Result<bool, String> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => {
            let time = time.as_secs_f64();
            if time <= token.expires {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(_) => {
            return Err(String::from("服务器时间存在致命错误"));
        }
    }
}

pub async fn clear_token(token_to_clear: &Token) -> Result<bool, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"student_id": token_to_clear.student_id_of_token.clone()};
                    let empty_token = Token::new_empty();
                    let update = doc! {"$set":{
                        "token":{
                            "token":empty_token.token.clone(),
                            "expires":empty_token.expires.clone(),
                            "student_id_of_token":empty_token.student_id_of_token.clone(),
                        }
                    }};
                    match collection.update_one(filter, update, None).await {
                        Ok(_) => {
                            return Ok(true);
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToClearToken(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToClearToken(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToClearToken(Box::new(e.kind.to_string()))));
        }
    }
}

pub async fn update_token(new_token: &Token) -> Result<bool, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"student_id": new_token.student_id_of_token.clone()};
                    let update = doc! {"$set":{
                        "token":{
                            "token":new_token.token.clone(),
                            "expires":new_token.expires.clone(),
                            "student_id_of_token":new_token.student_id_of_token.clone(),
                        }
                    }};
                    match collection.update_one(filter, update, None).await {
                        Ok(_) => {
                            return Ok(true);
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToUpdateToken(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToUpdateToken(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToUpdateToken(Box::new(e.kind.to_string()))));
        }
    }
}