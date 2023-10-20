use std::net::IpAddr;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde_json::json;

use crate::config;
use crate::token;
use crate::token::Token;
use crate::submit_login_info;

#[derive(Debug)]
pub struct FailedToSubmitAdminLoginInfo(Box<String>);

impl warp::reject::Reject for FailedToSubmitAdminLoginInfo {}

#[derive(Debug)]
pub struct FailedToGenerateToken(Box<String>);

impl warp::reject::Reject for FailedToGenerateToken {}


pub async fn fun_submit_admin_login_info(login_info: submit_login_info::LoginInfo) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("admins");
                    let filter = doc! {"student_id": login_info.student_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(user) => {
                                    if user.password == login_info.password {
                                        // 登录成功，返回新的token
                                        match Token::new(user.student_id.clone()) {
                                            Ok(token) => {
                                                match token::update_token(&token, "admins").await {
                                                    Ok(_) => {
                                                        match submit_login_info::update_sls_verification(user.student_id).await {
                                                            Ok(_) => {
                                                                let sth = json!({
                                                                    "status":config::API_STATUS_SUCCESS,
                                                                    "data":token
                                                                }); // 创造serde_json变量（类型叫Value）
                                                                let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                return Ok(sth_warp);
                                                            }
                                                            Err(e) => {
                                                                return Err(e);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        return Err(e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                return Err(warp::reject::custom(FailedToGenerateToken(Box::new(e.to_string()))));
                                            }
                                        }
                                    } else {
                                        let sth = json!({
                                            "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                            "reasons":"密码错误"
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
                            return Err(warp::reject::custom(FailedToSubmitAdminLoginInfo(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitAdminLoginInfo(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToSubmitAdminLoginInfo(Box::new(e.kind.to_string()))));
        }
    }
}