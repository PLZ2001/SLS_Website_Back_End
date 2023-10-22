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
pub struct SlsMemberProfileUpdate {
    pub email: String,
    pub phone_number: String,
    pub url: String,
    pub introduction: String,
    pub paper_years: Vec<String>,
    pub papers: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct FailedToSubmitSlsMemberProfileUpdate(Box<String>);

impl warp::reject::Reject for FailedToSubmitSlsMemberProfileUpdate {}

pub async fn fun_submit_sls_member_profile_update(sls_member_profile_update: SlsMemberProfileUpdate, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                            match collection.find(filter.clone(), find_options).await {
                                                                Ok(cursor) => {
                                                                    // Iterate over the results of the cursor.
                                                                    let mut cursor_enumerate = cursor.enumerate();
                                                                    'find_loop: loop {
                                                                        match cursor_enumerate.next().await {
                                                                            Some(find_result) => {
                                                                                match find_result.1 {
                                                                                    Ok(member) => {
                                                                                        let update = doc! {"$set":{
                                                                                            "email":sls_member_profile_update.email,
                                                                                            "phone_number":sls_member_profile_update.phone_number,
                                                                                            "url":sls_member_profile_update.url,
                                                                                            "introduction":sls_member_profile_update.introduction,
                                                                                            "paper_years":sls_member_profile_update.paper_years,
                                                                                            "papers":sls_member_profile_update.papers,
                                                                                        }};
                                                                                        match collection.update_one(filter, update, None).await {
                                                                                            Ok(_) => {
                                                                                                let sth = json!({
                                                                                                    "status":config::API_STATUS_SUCCESS,
                                                                                                    "data":member.name
                                                                                                }); // 创造serde_json变量（类型叫Value）
                                                                                                let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                                return Ok(sth_warp);
                                                                                            }
                                                                                            Err(e) => {
                                                                                                return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
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
                                                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
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
                                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitSlsMemberProfileUpdate(Box::new(e.kind.to_string()))));
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