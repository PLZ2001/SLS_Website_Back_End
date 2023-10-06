use std::net::IpAddr;
use crate::config;
use crate::token;
use serde_json::json;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct NewComment {
    pub object_commented_on: String,
    pub post_or_comment_id_commented_on: String,
    pub content: String,
    pub time: String,
    pub files: Vec<config::FILE>,
}

#[derive(Debug)]
pub struct FailedToSubmitNewComment(Box<String>);

impl warp::reject::Reject for FailedToSubmitNewComment {}

pub async fn fun_submit_new_comment(comment_id:String, new_comment: NewComment, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        let collection = db.collection::<config::COMMENT>("comments");
                                                        match new_comment.time.parse::<f64>() {
                                                            Ok(time) => {
                                                                let comment = config::COMMENT {
                                                                    comment_id: comment_id.clone(),
                                                                    content: new_comment.content,
                                                                    user_id: user.student_id,
                                                                    time: time,
                                                                    stat: config::STATS{watch:0,like:0,share:0,favorite:0,comment:0},
                                                                    files: new_comment.files,
                                                                    comment_ids: Vec::new(),
                                                                };
                                                                match collection.insert_one(comment.clone(), None).await {
                                                                    Ok(_) => {
                                                                        if new_comment.object_commented_on == "post" {
                                                                            let collection = db.collection::<config::POST>("posts");
                                                                            let filter = doc! {"post_id": new_comment.post_or_comment_id_commented_on.clone()};
                                                                            match collection.find_one(filter.clone(), None).await {
                                                                                Ok(find_result) => {
                                                                                    match find_result {
                                                                                        Some(post) => {
                                                                                            let mut comment_ids = post.comment_ids.clone();
                                                                                            comment_ids.push(comment_id);
                                                                                            let update = doc! {"$set":{
                                                                                                "comment_ids":comment_ids
                                                                                            }};
                                                                                            match collection.update_one(filter, update, None).await {
                                                                                                Ok(_) => {}
                                                                                                Err(e) => {
                                                                                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        None => {
                                                                                            let sth = json!({
                                                                                                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                                                "reasons":format!("评论对象不存在")
                                                                                            }); // 创造serde_json变量（类型叫Value）
                                                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                            return Ok(sth_warp);
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        } else if new_comment.object_commented_on == "comment" {
                                                                            let collection = db.collection::<config::COMMENT>("comments");
                                                                            let filter = doc! {"comment_id": new_comment.post_or_comment_id_commented_on.clone()};
                                                                            match collection.find_one(filter.clone(), None).await {
                                                                                Ok(find_result) => {
                                                                                    match find_result {
                                                                                        Some(comment) => {
                                                                                            let mut comment_ids = comment.comment_ids.clone();
                                                                                            comment_ids.push(comment_id);
                                                                                            let update = doc! {"$set":{
                                                                                                "comment_ids":comment_ids
                                                                                            }};
                                                                                            match collection.update_one(filter, update, None).await {
                                                                                                Ok(_) => {}
                                                                                                Err(e) => {
                                                                                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        None => {
                                                                                            let sth = json!({
                                                                                                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                                                "reasons":format!("评论对象不存在")
                                                                                            }); // 创造serde_json变量（类型叫Value）
                                                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                            return Ok(sth_warp);
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        } else {
                                                                            return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new("评论对象类型不存在".to_string()))));
                                                                        }
                                                                        let sth = json!({
                                                                            "status":config::API_STATUS_SUCCESS,
                                                                            "data":comment
                                                                        }); // 创造serde_json变量（类型叫Value）
                                                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                        return Ok(sth_warp);
                                                                    }
                                                                    Err(e) => {
                                                                        return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                                                    }
                                                                }
                                                            } Err(e) => {
                                                                return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.to_string()))));
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
                                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitNewComment(Box::new(e.kind.to_string()))));
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