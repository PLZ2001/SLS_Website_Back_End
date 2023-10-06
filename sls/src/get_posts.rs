use std::net::IpAddr;
use crate::config;
use serde_json::json;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct GetPostsConfig {
    pub pieces: String,
    pub sequence: String,
}

#[derive(Debug)]
pub struct FailedToGetPosts(Box<String>);

impl warp::reject::Reject for FailedToGetPosts {}

pub async fn fun_get_posts(get_posts_config: GetPostsConfig) -> Result<warp::reply::Json, warp::Rejection> {
    let mut posts = Vec::new();
    match get_posts_config.pieces.parse::<i32>() {
        Ok(pieces) => {
            match get_posts_config.sequence.parse::<i32>() {
                Ok(sequence) => {
                    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
                        Ok(client_options) => {
                            match Client::with_options(client_options) {
                                Ok(client) => {
                                    let db = client.database("forum");
                                    // Get a handle to a collection in the database.
                                    let collection = db.collection::<config::POST>("posts");
                                    let filter = doc! {};
                                    let find_options = FindOptions::builder().sort(doc! {"time":-1}).build();
                                    match collection.find(filter, find_options).await {
                                        Ok(cursor) => {
                                            // Iterate over the results of the cursor.
                                            let mut cursor_enumerate = cursor.enumerate();
                                            let mut cnt= 0;
                                            'find_loop: loop {
                                                match cursor_enumerate.next().await {
                                                    Some(find_result) => {
                                                        match find_result.1 {
                                                            Ok(post) => {
                                                                cnt = cnt + 1; // cnt表示搜索到第几条（从1开始计数）
                                                                if cnt <= (&sequence-1)*&pieces {
                                                                    continue 'find_loop;
                                                                } else if cnt <= &sequence*&pieces {
                                                                    posts.push(json!(post));
                                                                } else {
                                                                    break 'find_loop;
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.kind.to_string()))));
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
                                            return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.kind.to_string()))));
                                        }
                                    }

                                    let sth = json!({
                                        "status":config::API_STATUS_SUCCESS,
                                        "data":posts
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                                Err(e) => {
                                    return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToGetPosts(Box::new(e.to_string()))));
        }
    }
}