use std::net::IpAddr;
use crate::config;
use crate::token;
use serde_json::json;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct SignUpInfo {
    pub student_id: String,
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
                    let filter = doc! {"student_id": signup_info.student_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(_) => {
                                    let sth = json!({
                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                        "reasons":"该学号已存在账户"
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                                None => {
                                    let user = config::USER {
                                        student_id: signup_info.student_id,
                                        name: signup_info.name,
                                        grade: signup_info.grade,
                                        password: signup_info.password,
                                        token: token::Token::new_empty()
                                    };
                                    match collection.insert_one(user.clone(), None).await {
                                        Ok(_) => {
                                            let sth = json!({
                                                "status":config::API_STATUS_SUCCESS,
                                                "data":{
                                                    "student_id":user.student_id,
                                                    "name":user.name,
                                                    "grade":user.grade,
                                                    "password":user.password
                                                }
                                            }); // 创造serde_json变量（类型叫Value）
                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                            return Ok(sth_warp);
                                        }
                                        Err(e) => {
                                            return Err(warp::reject::custom(FailedToSubmitSignUpInfo(Box::new(e.kind.to_string()))));
                                        }
                                    }
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