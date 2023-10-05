use crate::config;
use serde_json::json;
use std::net::IpAddr;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;

#[derive(Debug)]
pub struct FailedToGetNameWithStudentId(Box<String>);

impl warp::reject::Reject for FailedToGetNameWithStudentId {}

pub async fn fun_get_name_with_student_id(student_id: String) -> Result<warp::reply::Json, warp::Rejection> {
    match ClientOptions::parse(format!("mongodb://{}:{}", IpAddr::from(config::MONGODB_URL), config::MONGODB_PORT)).await {
        Ok(client_options) => {
            match Client::with_options(client_options) {
                Ok(client) => {
                    let db = client.database("users");
                    // Get a handle to a collection in the database.
                    let collection = db.collection::<config::USER>("guests");
                    let filter = doc! {"student_id": student_id.clone()};
                    match collection.find_one(filter, None).await {
                        Ok(find_result) => {
                            match find_result {
                                Some(user) => {
                                    let sth = json!({
                                        "status":config::API_STATUS_SUCCESS,
                                        "data":user.name
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                                None => {
                                    let sth = json!({
                                        "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                        "reasons":format!("获取姓名失败")
                                    }); // 创造serde_json变量（类型叫Value）
                                    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                    return Ok(sth_warp);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetNameWithStudentId(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetNameWithStudentId(Box::new(e.kind.to_string()))));
                }
            }
        }
        Err(e) => {
            return Err(warp::reject::custom(FailedToGetNameWithStudentId(Box::new(e.kind.to_string()))));
        }
    }
}