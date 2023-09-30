use std::net::IpAddr;
use crate::config;
use serde_json::json;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use futures::StreamExt;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct SignUpInfo {
    pub name: String,
    pub grade: String,
    pub password: String,
}

#[derive(Debug)]
pub struct FailedToSubmitSignUpInfo(Box<String>);

impl warp::reject::Reject for FailedToSubmitSignUpInfo {}

pub async fn fun_submit_signup_info(signup_info: SignUpInfo) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"name": signup_info.name.clone()};
                    let find_options = FindOptions::builder().sort(doc! {"name": 1}).build();
                    match collection.find(filter, find_options).await {
                        Ok(cursor) => {
                            // Iterate over the results of the cursor.
                            let mut cursor_enumerate = cursor.enumerate();
                            'find_loop: loop {
                                match cursor_enumerate.next().await {
                                    Some(find_result) => {
                                        match find_result.1 {
                                            Ok(user) => {
                                                if user.name == signup_info.name && user.grade == signup_info.grade {
                                                    return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(String::from("该用户账号已存在")))));
                                                }
                                            }
                                            Err(e) => {
                                                return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
                                            }
                                        }
                                    }
                                    None => {
                                        break 'find_loop;
                                    }
                                }
                            }
                            let user = config::USER {
                                name: signup_info.name,
                                grade: signup_info.grade,
                                password: signup_info.password,
                            };
                            match collection.insert_one(user.clone(), None).await {
                                Ok(_) => {
                                    let sth = json!({
                                        "name":user.name,
                                        "grade":user.grade,
                                        "password":user.password}); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                                Err(e) => {
                                    return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
        }
    }
}