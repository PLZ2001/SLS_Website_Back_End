use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use crate::config;
use crate::token;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct NewSlsMember {
    pub name: String,
    pub student_id: String,
    pub description: String,
}

#[derive(Debug)]
pub struct FailedToSubmitNewSlsMember(Box<String>);

impl warp::reject::Reject for FailedToSubmitNewSlsMember {}

pub async fn fun_submit_new_sls_member(sls_member_category: String, new_sls_member: NewSlsMember, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        let filter = doc! {"student_id": new_sls_member.student_id.clone()};
                                                        let find_options = FindOptions::builder().sort(doc! {}).build();
                                                        match collection.find(filter.clone(), find_options).await {
                                                            Ok(cursor) => {
                                                                // Iterate over the results of the cursor.
                                                                let mut cursor_enumerate = cursor.enumerate();
                                                                'find_loop: loop {
                                                                    match cursor_enumerate.next().await {
                                                                        Some(find_result) => {
                                                                            match find_result.1 {
                                                                                Ok(_) => {
                                                                                    let sth = json!({
                                                                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                                        "reasons":"该山林寺成员已注册"
                                                                                    }); // 创造serde_json变量（类型叫Value）
                                                                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                    return Ok(sth_warp);
                                                                                }
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        }
                                                                        None => {
                                                                            break 'find_loop;
                                                                        }
                                                                    }
                                                                }
                                                                let new_file_name = format!("{}.png", new_sls_member.student_id.as_str());
                                                                let old_file_name = format!("{}.png", "default");
                                                                let new_file_path = format!("./{}{}/{}", config::DIR_STATIC, config::DIR_SLS_MEMBERS, new_file_name);
                                                                let old_file_path = format!("./{}{}/{}", config::DIR_STATIC, config::DIR_SLS_MEMBERS, old_file_name);
                                                                match fs::copy(old_file_path, new_file_path).await {
                                                                    Ok(_) => {
                                                                        let sls_member = config::SLSMEMBER {
                                                                            name: new_sls_member.name,
                                                                            description: new_sls_member.description,
                                                                            image: format!("{}.png", new_sls_member.student_id.as_str()),
                                                                            student_id: new_sls_member.student_id.clone(),
                                                                            introduction: String::new(),
                                                                            email: String::new(),
                                                                            phone_number: String::new(),
                                                                            papers: Vec::new(),
                                                                            paper_years: Vec::new(),
                                                                            url: format!("http://{}:{}/sls_member/{}", config::FRONT_URL, config::FRONT_PORT, new_sls_member.student_id),
                                                                        };
                                                                        match collection.insert_one(sls_member.clone(), None).await {
                                                                            Ok(_) => {
                                                                                let sth = json!({
                                                                        "status":config::API_STATUS_SUCCESS,
                                                                        "data":sls_member
                                                                        }); // 创造serde_json变量（类型叫Value）
                                                                                let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                return Ok(sth_warp);
                                                                            }
                                                                            Err(e) => {
                                                                                return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind().to_string()))));
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
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
                                                    return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitNewSlsMember(Box::new(e.kind.to_string()))));
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