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
pub struct LoginInfo {
    pub student_id: String,
    pub password: String,
}

#[derive(Debug)]
pub struct FailedToSubmitLoginInfo(Box<String>);

impl warp::reject::Reject for FailedToSubmitLoginInfo {}

#[derive(Debug)]
pub struct FailedToGenerateToken(Box<String>);

impl warp::reject::Reject for FailedToGenerateToken {}

#[derive(Debug)]
pub struct FailedToVerifySlsMember(Box<String>);

impl warp::reject::Reject for FailedToVerifySlsMember {}

#[derive(Debug)]
pub struct FailedToUpdateSlsVerification(Box<String>);

impl warp::reject::Reject for FailedToUpdateSlsVerification {}

async fn sls_members_verification(student_id: String) -> Result<bool, warp::Rejection> {
    let sls_member_categories = vec!["teachers", "students", "graduates"];
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("sls_members");
                    for collection_name in sls_member_categories {
                        // Get a handle to a collection in the database.
                        let collection = db.collection::<config::SLSMEMBER>(collection_name);
                        let filter = doc! {"student_id": student_id.clone()};
                        let find_options = FindOptions::builder().sort(doc! {}).build();
                        match collection.find(filter, find_options).await {
                            Ok(cursor) => {
                                // Iterate over the results of the cursor.
                                let mut cursor_enumerate = cursor.enumerate();
                                'find_loop: loop {
                                    match cursor_enumerate.next().await {
                                        Some(find_result) => {
                                            match find_result.1 {
                                                Ok(_) => {
                                                    return Ok(true);
                                                }
                                                Err(e) => {
                                                    return Err(warp::reject::custom(FailedToVerifySlsMember(Box::new(e.kind.to_string()))));
                                                }
                                            }
                                        }
                                        None => {
                                            break 'find_loop;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(warp::reject::custom(FailedToVerifySlsMember(Box::new(e.kind.to_string()))));
                            }
                        }
                    }
                    return Ok(false);
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToVerifySlsMember(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToVerifySlsMember(Box::new(e.kind.to_string()))));
        }
    }
}

async fn update_sls_verification(student_id: String) -> Result<bool, warp::Rejection> {
    match sls_members_verification(student_id.clone()).await {
        Ok(verification) => {
            match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
                Ok(client_options) => {
                    match Client::with_options(client_options) {
                        Ok(client) => {
                            let db = client.database("users");
                            // Get a handle to a collection in the database.
                            let collection = db.collection::<config::USER>("guests");
                            let filter = doc! {"student_id": student_id.clone()};
                            let update = doc! {"$set":{
                                "sls_verification":verification,
                            }};
                            match collection.update_one(filter, update, None).await {
                                Ok(_) => {
                                    return Ok(true);
                                }
                                Err(e) => {
                                    return Err(warp::reject::custom(FailedToUpdateSlsVerification(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToUpdateSlsVerification(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToUpdateSlsVerification(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn fun_submit_login_info(login_info: LoginInfo) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"student_id": login_info.student_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(user) => {
                                    if user.password == login_info.password {
                                        // 登录成功，返回新的token
                                        match Token::new(user.student_id.clone()) {
                                            Ok(token) => {
                                                match token::update_token(&token).await {
                                                    Ok(_) => {
                                                        match update_sls_verification(user.student_id).await {
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
                            return Err(warp::reject::custom(FailedToSubmitLoginInfo(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitLoginInfo(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToSubmitLoginInfo(Box::new(e.kind.to_string()))));
        }
    }
}