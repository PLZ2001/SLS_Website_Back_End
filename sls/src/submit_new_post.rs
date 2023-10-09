use std::net::IpAddr;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;
use crate::token;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub time: String,
    pub files: Vec<config::FILE>,
}

#[derive(Debug)]
pub struct FailedToSubmitNewPost(Box<String>);

impl warp::reject::Reject for FailedToSubmitNewPost {}

pub async fn fun_submit_new_post(post_id: String, new_post: NewPost, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        let db = client.database("forum");
                                                        // Get a handle to a collection in the database.
                                                        let collection = db.collection::<config::POST>("posts");
                                                        match new_post.time.parse::<f64>() {
                                                            Ok(time) => {
                                                                let post = config::POST {
                                                                    post_id: post_id,
                                                                    title: new_post.title,
                                                                    content: new_post.content,
                                                                    user_id: user.student_id,
                                                                    time: time,
                                                                    stat: config::STATS { watch: 0, like: 0, favorite: 0, comment: 0 },
                                                                    files: new_post.files,
                                                                    comment_ids: Vec::new(),
                                                                    watch_ids: Vec::new(),
                                                                    like_ids: Vec::new(),
                                                                    favorite_ids: Vec::new(),
                                                                };
                                                                match collection.insert_one(post.clone(), None).await {
                                                                    Ok(_) => {
                                                                        let sth = json!({
                                                                            "status":config::API_STATUS_SUCCESS,
                                                                            "data":post
                                                                        }); // 创造serde_json变量（类型叫Value）
                                                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                        return Ok(sth_warp);
                                                                    }
                                                                    Err(e) => {
                                                                        return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e.kind.to_string()))));
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e.to_string()))));
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
                                                    return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitNewPost(Box::new(e.kind.to_string()))));
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