use crate::config;
use crate::token;
use serde_json::json;
use std::net::IpAddr;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;

#[derive(Debug)]
pub struct FailedToGetUserName(Box<String>);

impl warp::reject::Reject for FailedToGetUserName {}

pub async fn fun_get_user_name(token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
    match token {
        Some(token) => {
            match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
                Ok(client_options) => {
                    match Client::with_options(client_options) {
                        Ok(client) => {
                            let db = client.database("users");
                            // Get a handle to a collection in the database.
                            let collection = db.collection::<config::USER>("guests");
                            let filter = doc! {"token.token": token.clone()};
                            match collection.find_one(filter, None).await {
                                Ok(find_result) => {
                                    match find_result {
                                        Some(user) => {
                                            match token::validate_token(&user.token).await {
                                                Ok(validation_result) => {
                                                    if validation_result {
                                                        let sth = json!({
                                                            "status":config::API_STATUS_SUCCESS,
                                                            "data":user.name
                                                        }); // 创造serde_json变量（类型叫Value）
                                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                        return Ok(sth_warp);
                                                    } else {
                                                        match token::clear_token(&user.token).await {
                                                            Ok(_) => {
                                                                let sth = json!({
                                                                    "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                    "reasons":"用户token已过期，请重新登录"
                                                                }); // 创造serde_json变量（类型叫Value）
                                                                let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                return Ok(sth_warp);
                                                            }
                                                            Err(e) => {
                                                                return Err(e);
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    return Err(warp::reject::custom(FailedToGetUserName(Box::new(e))));
                                                }
                                            }
                                        }
                                        None => {
                                            let sth = json!({
                                                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                "reasons":format!("token无效")
                                            }); // 创造serde_json变量（类型叫Value）
                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                            return Ok(sth_warp);
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(warp::reject::custom(FailedToGetUserName(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetUserName(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetUserName(Box::new(e.kind.to_string()))));
                }
            }
        }
        None => {
            let sth = json!({
                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                "reasons":"无token"
            }); // 创造serde_json变量（类型叫Value）
            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
            return Ok(sth_warp);
        }
    }
}