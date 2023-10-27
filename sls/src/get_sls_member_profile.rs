use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde_json::json;

use crate::config;
use crate::token;

#[derive(Debug)]
pub struct FailedToGetSlsMemberProfile(Box<String>);

impl warp::reject::Reject for FailedToGetSlsMemberProfile {}

pub async fn fun_get_sls_member_profile(token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        let sls_member_categories = vec!["teachers", "students", "graduates"];
                                                        let db = client.database("sls_members");
                                                        for collection_name in sls_member_categories {
                                                            // Get a handle to a collection in the database.
                                                            let collection = db.collection::<config::SLSMEMBER>(collection_name);
                                                            let filter = doc! {"student_id": user.student_id.clone()};
                                                            let find_options = FindOptions::builder().sort(doc! {}).build();
                                                            match collection.find(filter, find_options).await {
                                                                Ok(cursor) => {
                                                                    // Iterate over the results of the cursor.
                                                                    let mut cursor_enumerate = cursor.enumerate();
                                                                    'find_loop: loop {
                                                                        match cursor_enumerate.next().await {
                                                                            Some(find_result) => {
                                                                                match find_result.1 {
                                                                                    Ok(member) => {
                                                                                        let mut _member = member.clone();
                                                                                        _member.image = format!("http://{}:{}/{}{}", config::SERVER_URL, config::SERVER_PORT, config::DIR_SLS_MEMBERS, member.image);
                                                                                        let sth = json!({
                                                                                            "status":config::API_STATUS_SUCCESS,
                                                                                            "data":_member
                                                                                        }); // 创造serde_json变量（类型叫Value）
                                                                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                        return Ok(sth_warp);
                                                                                    }
                                                                                    Err(e) => {
                                                                                        return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e.kind.to_string()))));
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
                                                                    return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e.kind.to_string()))));
                                                                }
                                                            }
                                                        }
                                                        let sth = json!({
                                                            "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                            "reasons":"用户非山林寺成员"
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
                                                    return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetSlsMemberProfile(Box::new(e.kind.to_string()))));
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