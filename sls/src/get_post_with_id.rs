use std::net::IpAddr;

use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde_json::json;

use crate::config;

#[derive(Debug)]
pub struct FailedToGetPostWithId(Box<String>);

impl warp::reject::Reject for FailedToGetPostWithId {}

pub async fn fun_get_post_with_id(post_id: String) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("forum");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::POST>("posts");
                    let filter = doc! {"post_id": post_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(post) => {
                                    let sth = json!({
                                        "status":config::API_STATUS_SUCCESS,
                                        "data":post
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                                None => {
                                    let sth = json!({
                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                        "reasons":format!("该帖子不存在")
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetPostWithId(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetPostWithId(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToGetPostWithId(Box::new(e.kind.to_string()))));
        }
    }
}