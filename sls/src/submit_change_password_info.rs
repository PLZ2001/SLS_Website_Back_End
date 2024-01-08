use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;
use crate::token;
use crate::token::Token;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct ChangePasswordInfo {
    pub student_id: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug)]
pub struct FailedToSubmitChangePasswordInfo(Box<String>);

impl warp::reject::Reject for FailedToSubmitChangePasswordInfo {}

pub async fn fun_submit_change_password_info(change_password_info: ChangePasswordInfo) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"student_id": change_password_info.student_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(user) => {
                                    if user.password == change_password_info.old_password {
                                        // 匹配成功，可以修改密码
                                        let filter = doc! {"student_id": change_password_info.student_id.clone()};
                                        let update = doc! {"$set":{
                                            "password": change_password_info.new_password,
                                        }};
                                        match collection.update_one(filter, update, None).await {
                                            Ok(_) => {
                                                let sth = json!({
                                                                    "status":config::API_STATUS_SUCCESS,
                                                                    "data":""
                                                                }); // 创造serde_json变量（类型叫Value）
                                                let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                return Ok(sth_warp);
                                            }
                                            Err(e) => {
                                                return Err(warp::reject::custom(FailedToSubmitChangePasswordInfo(Box::new(e.kind.to_string()))));
                                            }
                                        }
                                    } else {
                                        let sth = json!({
                                            "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                            "reasons":"旧密码错误"
                                        }); // 创造serde_json变量（类型叫Value）
                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                        return Ok(sth_warp);
                                    }
                                }
                                None => {
                                    let sth = json!({
                                            "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                            "reasons":"账户未注册"
                                        }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitChangePasswordInfo(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitChangePasswordInfo(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToSubmitChangePasswordInfo(Box::new(e.kind.to_string()))));
        }
    }
}