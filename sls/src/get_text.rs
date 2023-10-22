use std::net::IpAddr;

use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde_json::json;

use crate::config;

#[derive(Debug)]
pub struct FailedToGetText(Box<String>);

impl warp::reject::Reject for FailedToGetText {}

pub async fn fun_get_text(category: String) -> Result<warp::reply::Json, warp::Rejection> {
    let mut texts = Vec::new();
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("text");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::TEXT>(&category);
                    let filter = doc! {};
                    let find_options = FindOptions::builder().sort(doc! {"time":-1}).build();
                    match collection.find(filter, find_options).await {
                        Ok(cursor) => {
                            // Iterate over the results of the cursor.
                            let mut cursor_enumerate = cursor.enumerate();
                            'find_loop: loop {
                                match cursor_enumerate.next().await {
                                    Some(find_result) => {
                                        match find_result.1 {
                                            Ok(text) => {
                                                texts.push(text);
                                            }
                                            Err(e) => {
                                                return Err(warp::reject::custom(FailedToGetText(Box::new(e.kind.to_string()))));
                                            }
                                        }
                                    }
                                    None => {
                                        break 'find_loop;
                                    }
                                }
                            }
                            let sth = json!({
                                "status":config::API_STATUS_SUCCESS,
                                "data":texts,
                            }); // 创造serde_json变量（类型叫Value）
                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                            return Ok(sth_warp);
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetText(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetText(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToGetText(Box::new(e.kind.to_string()))));
        }
    }
}