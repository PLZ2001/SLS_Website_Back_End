use std::net::IpAddr;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;
use crate::token;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Action {
    pub action_category: String,
    pub object_acted_on: String,
    pub post_or_comment_id_acted_on: String,
}

#[derive(Debug)]
pub struct FailedToSubmitAnAction(Box<String>);

impl warp::reject::Reject for FailedToSubmitAnAction {}

pub async fn fun_submit_an_action(action: Action, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        if action.object_acted_on == "post" {
                                                            let collection = db.collection::<config::POST>("posts");
                                                            let filter = doc! {"post_id": action.post_or_comment_id_acted_on.clone()};
                                                            match collection.find_one(filter.clone(), None).await {
                                                                Ok(find_result) => {
                                                                    match find_result {
                                                                        Some(post) => {
                                                                            let mut stat = post.stat.clone();
                                                                            let mut watch_ids = post.watch_ids.clone();
                                                                            let mut like_ids = post.like_ids.clone();
                                                                            let mut favorite_ids = post.favorite_ids.clone();
                                                                            match action.action_category.as_str() {
                                                                                "watch" => {
                                                                                    stat.watch = stat.watch + 1;
                                                                                    watch_ids.push(user.student_id);
                                                                                }
                                                                                "like" => {
                                                                                    match post.like_ids.iter().position(|x| x == &user.student_id) {
                                                                                        Some(index) => {
                                                                                            like_ids.remove(index);
                                                                                            stat.like = stat.like - 1;
                                                                                        }
                                                                                        None => {
                                                                                            like_ids.push(user.student_id);
                                                                                            stat.like = stat.like + 1;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                "favorite" => {
                                                                                    match post.favorite_ids.iter().position(|x| x == &user.student_id) {
                                                                                        Some(index) => {
                                                                                            favorite_ids.remove(index);
                                                                                            stat.favorite = stat.favorite - 1;
                                                                                        }
                                                                                        None => {
                                                                                            favorite_ids.push(user.student_id);
                                                                                            stat.favorite = stat.favorite + 1;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                _ => {
                                                                                    let sth = json!({
                                                                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                                        "reasons":format!("不存在该类型动作")
                                                                                    }); // 创造serde_json变量（类型叫Value）
                                                                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                    return Ok(sth_warp);
                                                                                }
                                                                            }
                                                                            let update = doc! {"$set":{
                                                                                "stat":{
                                                                                    "watch":stat.watch,
                                                                                    "like":stat.like,
                                                                                    "favorite":stat.favorite,
                                                                                    "comment":stat.comment,
                                                                                },
                                                                                "watch_ids":watch_ids,
                                                                                "like_ids":like_ids,
                                                                                "favorite_ids":favorite_ids,
                                                                            }};
                                                                            match collection.update_one(filter, update, None).await {
                                                                                Ok(_) => {}
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
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
                                                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
                                                                }
                                                            }
                                                        } else if action.object_acted_on == "comment" {
                                                            let collection = db.collection::<config::COMMENT>("comments");
                                                            let filter = doc! {"comment_id": action.post_or_comment_id_acted_on.clone()};
                                                            match collection.find_one(filter.clone(), None).await {
                                                                Ok(find_result) => {
                                                                    match find_result {
                                                                        Some(comment) => {
                                                                            let mut stat = comment.stat.clone();
                                                                            let mut watch_ids = comment.watch_ids.clone();
                                                                            let mut like_ids = comment.like_ids.clone();
                                                                            let mut favorite_ids = comment.favorite_ids.clone();
                                                                            match action.action_category.as_str() {
                                                                                "watch" => {
                                                                                    stat.watch = stat.watch + 1;
                                                                                    watch_ids.push(user.student_id);
                                                                                }
                                                                                "like" => {
                                                                                    match comment.like_ids.iter().position(|x| x == &user.student_id) {
                                                                                        Some(index) => {
                                                                                            like_ids.remove(index);
                                                                                            stat.like = stat.like - 1;
                                                                                        }
                                                                                        None => {
                                                                                            like_ids.push(user.student_id);
                                                                                            stat.like = stat.like + 1;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                "favorite" => {
                                                                                    match comment.favorite_ids.iter().position(|x| x == &user.student_id) {
                                                                                        Some(index) => {
                                                                                            favorite_ids.remove(index);
                                                                                            stat.favorite = stat.favorite - 1;
                                                                                        }
                                                                                        None => {
                                                                                            favorite_ids.push(user.student_id);
                                                                                            stat.favorite = stat.favorite + 1;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                _ => {
                                                                                    let sth = json!({
                                                                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                                        "reasons":format!("不存在该类型动作")
                                                                                    }); // 创造serde_json变量（类型叫Value）
                                                                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                                    return Ok(sth_warp);
                                                                                }
                                                                            }
                                                                            let update = doc! {"$set":{
                                                                                "stat":{
                                                                                    "watch":stat.watch,
                                                                                    "like":stat.like,
                                                                                    "favorite":stat.favorite,
                                                                                    "comment":stat.comment,
                                                                                },
                                                                                "watch_ids":watch_ids,
                                                                                "like_ids":like_ids,
                                                                                "favorite_ids":favorite_ids,
                                                                            }};
                                                                            match collection.update_one(filter, update, None).await {
                                                                                Ok(_) => {}
                                                                                Err(e) => {
                                                                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
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
                                                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
                                                                }
                                                            }
                                                        } else {
                                                            return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new("评论对象类型不存在".to_string()))));
                                                        }
                                                        let sth = json!({
                                                            "status":config::API_STATUS_SUCCESS,
                                                            "data":action
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
                                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitAnAction(Box::new(e.kind.to_string()))));
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