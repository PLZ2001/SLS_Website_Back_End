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
pub struct CommentRemoving {
    pub comment_ids: Vec<String>,
}

#[derive(Debug)]
pub struct FailedToSubmitCommentRemoving(Box<String>);

impl warp::reject::Reject for FailedToSubmitCommentRemoving {}

pub async fn fun_submit_comment_removing(comment_removing: CommentRemoving, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        let db = client.database("forum");
                                                        // Get a handle to a collection in the database.
                                                        let collection = db.collection::<config::COMMENT>("comments");
                                                        let filter = doc! {"comment_id": {"$in":comment_removing.comment_ids.clone()}};
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
                                                                                    return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
                                                                                }
                                                                            }
                                                                        }
                                                                        None => {
                                                                            break 'find_loop;
                                                                        }
                                                                    }
                                                                }
                                                                if counter == comment_removing.comment_ids.len() {
                                                                    let filter = doc! {"comment_id": {"$in":comment_removing.comment_ids.clone()}};
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
                                                                            return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
                                                                        }
                                                                    }
                                                                } else {
                                                                    let sth = json!({
                                                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                        "reasons":"不能删除不存在的评论"
                                                                    }); // 创造serde_json变量（类型叫Value）
                                                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                    return Ok(sth_warp);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
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
                                                    return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitCommentRemoving(Box::new(e.kind.to_string()))));
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