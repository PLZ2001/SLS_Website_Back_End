use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct GetFavoritePostsWithStudentIdConfig {
    pub pieces: String,
    pub sequence: String,
}

#[derive(Debug)]
pub struct FailedToGetFavoritePostsWithStudentId(Box<String>);

impl warp::reject::Reject for FailedToGetFavoritePostsWithStudentId {}

pub async fn fun_get_favorite_posts_with_student_id(student_id: String, get_favorite_posts_with_student_id_config: GetFavoritePostsWithStudentIdConfig) -> Result<warp::reply::Json, warp::Rejection> {
    let mut posts = Vec::new();
    match get_favorite_posts_with_student_id_config.pieces.parse::<i32>() {
        Ok(pieces) => {
            match get_favorite_posts_with_student_id_config.sequence.parse::<i32>() {
                Ok(sequence) => {
                    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
                        Ok(client_options) => {
                            match Client::with_options(client_options) {
                                Ok(client) => {
                                    let db = client.database("forum");
                                    // Get a handle to a collection in the database.
                                    let collection = db.collection::<config::POST>("posts");
                                    let filter = doc! {"favorite_ids": student_id};
                                    let find_options = FindOptions::builder().sort(doc! {"time":-1}).build();
                                    match collection.find(filter, find_options).await {
                                        Ok(cursor) => {
                                            // Iterate over the results of the cursor.
                                            let mut cursor_enumerate = cursor.enumerate();
                                            let mut cnt = 0;
                                            'find_loop: loop {
                                                match cursor_enumerate.next().await {
                                                    Some(find_result) => {
                                                        match find_result.1 {
                                                            Ok(post) => {
                                                                cnt = cnt + 1; // cnt表示搜索到第几条（从1开始计数）
                                                                if cnt <= (&sequence - 1) * &pieces {
                                                                    continue 'find_loop;
                                                                } else if cnt <= &sequence * &pieces {
                                                                    posts.push(json!(post));
                                                                } else {
                                                                    break 'find_loop;
                                                                }
                                                            }
                                                            Err(e) => {
                                                                return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.kind.to_string()))));
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
                                            return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.kind.to_string()))));
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
                                    return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToGetFavoritePostsWithStudentId(Box::new(e.to_string()))));
        }
    }
}