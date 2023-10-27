use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;
use crate::token;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct SlsMemberMoving {
    pub sls_member_category_target: String,
    pub student_ids: Vec<String>,
}

#[derive(Debug)]
pub struct FailedToSubmitSlsMemberMoving(Box<String>);

impl warp::reject::Reject for FailedToSubmitSlsMemberMoving {}

pub async fn fun_submit_sls_member_moving(sls_member_category: String, sls_member_moving: SlsMemberMoving, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
    match token {
        Some(token) => {
            match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
                Ok(client_options) => {
                    match Client::with_options(client_options) {
                        Ok(client) => {
                            let db = client.database("users");
                            // Get a handle to a collection in the database.
                            let collection = db.collection::<config::USER>("admins");
                            let filter = doc! {"token.token": token.clone()};
                            match collection.find_one(filter, None).await {
                                Ok(find_result) => {
                                    match find_result {
                                        Some(user) => {
                                            match token::validate_token(&user.token).await {
                                                Ok(validation_result) => {
                                                    if validation_result {
                                                        let db = client.database("sls_members");
                                                        // Get a handle to a collection in the database.
                                                        let collection = db.collection::<config::SLSMEMBER>(&sls_member_category);
                                                        let filter = doc! {"student_id": {"$in":sls_member_moving.student_ids.clone()}};
                                                        let find_options = FindOptions::builder().sort(doc! {}).build();
                                                        match collection.find(filter.clone(), find_options).await {
                                                            Ok(cursor) => {
                                                                // Iterate over the results of the cursor.
                                                                let mut cursor_enumerate = cursor.enumerate();
                                                                let mut counter = 0;
                                                                'find_loop: loop {
                                                                    match cursor_enumerate.next().await {
                                                                        Some(find_result) => {
                                                                            match find_result.1 {
                                                                                Ok(_) => {
                                                                                    counter = counter + 1;
                                                                                }
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        }
                                                                        None => {
                                                                            break 'find_loop;
                                                                        }
                                                                    }
                                                                }
                                                                if counter == sls_member_moving.student_ids.len() {
                                                                    let filter = doc! {"student_id": {"$in":sls_member_moving.student_ids.clone()}};
                                                                    let find_options = FindOptions::builder().sort(doc! {}).build();
                                                                    let mut members = Vec::new();
                                                                    match collection.find(filter.clone(), find_options).await {
                                                                        Ok(cursor) => {
                                                                            // Iterate over the results of the cursor.
                                                                            let mut cursor_enumerate = cursor.enumerate();
                                                                            'find_loop: loop {
                                                                                match cursor_enumerate.next().await {
                                                                                    Some(find_result) => {
                                                                                        match find_result.1 {
                                                                                            Ok(member) => {
                                                                                                members.push(member);
                                                                                            }
                                                                                            Err(e) => {
                                                                                                return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    None => {
                                                                                        break 'find_loop;
                                                                                    }
                                                                                }
                                                                            }
                                                                            let collection = db.collection::<config::SLSMEMBER>(&sls_member_moving.sls_member_category_target);
                                                                            match collection.insert_many(members, None).await {
                                                                                Ok(_) => {
                                                                                    let collection = db.collection::<config::SLSMEMBER>(&sls_member_category);
                                                                                    match collection.delete_many(filter, None).await {
                                                                                        Ok(_) => {
                                                                                            let sth = json!({
                                                                                                "status":config::API_STATUS_SUCCESS,
                                                                                                "data":""
                                                                                            }); // 创造serde_json变量（类型叫Value）
                                                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                            return Ok(sth_warp);
                                                                                        }
                                                                                        Err(e) => {
                                                                                            return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                                        }
                                                                    }
                                                                } else {
                                                                    let sth = json!({
                                                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                        "reasons":"不能转移不存在的山林寺成员"
                                                                    }); // 创造serde_json变量（类型叫Value）
                                                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                    return Ok(sth_warp);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                                            }
                                                        }
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
                                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e))));
                                                }
                                            }
                                        }
                                        None => {
                                            let sth = json!({
                                                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                "reasons":format!("请重新登录")
                                            }); // 创造serde_json变量（类型叫Value）
                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                            return Ok(sth_warp);
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitSlsMemberMoving(Box::new(e.kind.to_string()))));
                }
            }
        }
        None => {
            let sth = json!({
                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                "reasons":"请登录后再试"
            }); // 创造serde_json变量（类型叫Value）
            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
            return Ok(sth_warp);
        }
    }
}