use std::net::IpAddr;

use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use serde_json::json;
use tokio::task;
use warp::multipart::FormData;

use crate::config;
use crate::submit_files;
use crate::token;

#[derive(Debug)]
pub struct FailedToSubmitImages(Box<String>);

impl warp::reject::Reject for FailedToSubmitImages {}

pub async fn fun_submit_images(folder_category: String, form: FormData, token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
    let folder;
    match folder_category.as_str() {
        "photo_wall" => {
            folder = config::DIR_PHOTO_WALL;
        }
        "annual" => {
            folder = config::DIR_ANNUAL;
        }
        _ => {
            folder = config::DIR_TEMP;
        }
    }
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
                                                        task::spawn(async move {
                                                            let mut parts = form.into_stream();
                                                            'find_loop: loop {
                                                                match parts.next().await {
                                                                    Some(p) => {
                                                                        match p {
                                                                            Ok(p) => {
                                                                                let file_name = format!("{}", p.name());
                                                                                let file_path = format!("./{}{}/{}", config::DIR_STATIC, folder, file_name);
                                                                                match submit_files::save_part_to_file(file_path, p).await {
                                                                                    Ok(_) => {}
                                                                                    Err(e) => {
                                                                                        return Err(e);
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                return Err(warp::reject::custom(FailedToSubmitImages(Box::new(e.to_string()))));
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
                                                                "data":""
                                                            }); // 创造serde_json变量（类型叫Value）
                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                            return Ok(sth_warp);
                                                        });
                                                        let sth = json!({
                                                                "status":config::API_STATUS_SUCCESS,
                                                                "data":""
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
                                                    return Err(warp::reject::custom(FailedToSubmitImages(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToSubmitImages(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToSubmitImages(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToSubmitImages(Box::new(e.kind.to_string()))));
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