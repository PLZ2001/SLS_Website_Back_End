use std::collections::HashMap;
use std::fs::metadata;
use std::net::IpAddr;
use walkdir::{WalkDir};
use std::os::windows::fs::MetadataExt;
use std::path::Path;

use mongodb::bson::doc;
use mongodb::Client;
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha256::digest;
use tokio::fs;
use chrono::DateTime;
use chrono::offset::Local;

use crate::{config, token};

#[derive(Debug)]
pub struct FailedToGetFsMap(Box<String>);

impl warp::reject::Reject for FailedToGetFsMap {}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct FileObject {
    pub id: String,
    pub name: String,
    pub isDir: bool,
    pub childrenIds: Vec<String>,
    pub parentId: String,
    pub childrenCount: usize,
    pub size: u64,
    pub modDate: String,
    pub thumbnailUrl: String,
}

fn is_hidden(
    file_path: &Path
) -> bool
{
    return match metadata(file_path) {
        Ok(metadata) => {
            let attributes = metadata.file_attributes();
            if (attributes & 0x2) > 0 {
                true
            } else {
                false
            }
        }
        Err(_) => {
            true
        }
    };
}

pub async fn fun_get_fsmap(token: Option<String>) -> Result<warp::reply::Json, warp::Rejection> {
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
                                                        if user.sls_verification {
                                                            let mut fsmap: HashMap<String, FileObject> = HashMap::new();
                                                            let mut fsmap_depth_map: HashMap<usize, Vec<FileObject>> = HashMap::new();
                                                            for entry_result in WalkDir::new(format!("{}{}", config::DIR_STATIC, config::DIR_FTP)).into_iter().filter_entry(|e| !is_hidden(e.path())) {
                                                                match entry_result {
                                                                    Ok (entry) => {
                                                                        if entry.file_type().is_file() {
                                                                            if let Some(file_name) = entry.path().file_name() {
                                                                                let file_name_str = file_name.to_string_lossy().to_string();
                                                                                let file_path_str = entry.path().to_string_lossy().to_string().replace("\\", "/");
                                                                                let id = digest(file_path_str.clone());
                                                                                let mut parentId= String::new();
                                                                                if entry.depth() > 0 {
                                                                                    if let Some(parent_file) = entry.path().parent() {
                                                                                        if let Some(parent_file_name) = parent_file.file_name() {
                                                                                            let parent_file_name_str = parent_file_name.to_string_lossy().to_string();
                                                                                            if let Some(vector) = fsmap_depth_map.get(&(entry.depth()-1)) {
                                                                                                let mut parent_file_objects = vector.iter().filter(|&x| x.isDir.clone() && x.name == parent_file_name_str);
                                                                                                'filter_loop: loop {
                                                                                                    match parent_file_objects.next() {
                                                                                                        Some(parent_file_object) => {
                                                                                                            parentId = parent_file_object.id.clone();
                                                                                                            if let Some(parent_file_object) = fsmap.get(parentId.clone().as_str()) {
                                                                                                                let mut parent_file_object_update = parent_file_object.clone();
                                                                                                                parent_file_object_update.childrenCount = parent_file_object_update.childrenCount + 1;
                                                                                                                parent_file_object_update.childrenIds.push(id.clone());
                                                                                                                fsmap.insert(parentId.clone(), parent_file_object_update.clone());
                                                                                                            }
                                                                                                            break 'filter_loop;
                                                                                                        }
                                                                                                        None => {}
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                match fs::metadata(entry.path()).await {
                                                                                    Ok(metadata) => {
                                                                                        match metadata.modified() {
                                                                                            Ok(modDate) => {
                                                                                                let datetime: DateTime<Local> = modDate.into();
                                                                                                let file_object = FileObject {
                                                                                                    id:id.clone(),
                                                                                                    name:file_name_str.clone(),
                                                                                                    isDir: false,
                                                                                                    childrenIds: Vec::new(),
                                                                                                    parentId: parentId.clone(),
                                                                                                    childrenCount: 0,
                                                                                                    size: metadata.file_size(),
                                                                                                    modDate: datetime.format("%Y/%m/%d %T").to_string(),
                                                                                                    thumbnailUrl: format!("http://{}:{}/{}", config::SERVER_URL, config::SERVER_PORT, &file_path_str[config::DIR_STATIC.len()..]),
                                                                                                };
                                                                                                fsmap.insert(id.clone(), file_object.clone());
                                                                                                match fsmap_depth_map.get(&(entry.depth())) {
                                                                                                    Some(vector) => {
                                                                                                        let mut vector_new = vector.clone();
                                                                                                        vector_new.push(file_object.clone());
                                                                                                        fsmap_depth_map.insert(entry.depth(), vector_new);
                                                                                                    }
                                                                                                    None => {
                                                                                                        let mut vector_new = Vec::new();
                                                                                                        vector_new.push(file_object.clone());
                                                                                                        fsmap_depth_map.insert(entry.depth(), vector_new);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(e) => {
                                                                                                return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind().to_string()))));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind().to_string()))));
                                                                                    }
                                                                                }
                                                                            }
                                                                        } else if entry.file_type().is_dir() {
                                                                            if let Some(file_name) = entry.path().file_name() {
                                                                                let file_name_str = file_name.to_string_lossy().to_string();
                                                                                let file_path_str = entry.path().to_string_lossy().to_string().replace("\\", "/");
                                                                                let id = digest(file_path_str.clone());
                                                                                let mut parentId= String::new();
                                                                                if entry.depth() > 0 {
                                                                                    if let Some(parent_file) = entry.path().parent() {
                                                                                        if let Some(parent_file_name) = parent_file.file_name() {
                                                                                            let parent_file_name_str = parent_file_name.to_string_lossy().to_string();
                                                                                            if let Some(vector) = fsmap_depth_map.get(&(entry.depth()-1)) {
                                                                                                let mut parent_file_objects = vector.iter().filter(|&x| x.isDir.clone() && x.name == parent_file_name_str);
                                                                                                'filter_loop: loop {
                                                                                                    match parent_file_objects.next() {
                                                                                                        Some(parent_file_object) => {
                                                                                                            parentId = parent_file_object.id.clone();
                                                                                                            if let Some(parent_file_object) = fsmap.get(parentId.clone().as_str()) {
                                                                                                                let mut parent_file_object_update = parent_file_object.clone();
                                                                                                                parent_file_object_update.childrenCount = parent_file_object_update.childrenCount + 1;
                                                                                                                parent_file_object_update.childrenIds.push(id.clone());
                                                                                                                fsmap.insert(parentId.clone(), parent_file_object_update.clone());
                                                                                                            }
                                                                                                            break 'filter_loop;
                                                                                                        }
                                                                                                        None => {}
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                match fs::metadata(entry.path()).await {
                                                                                    Ok(metadata) => {
                                                                                        match metadata.modified() {
                                                                                            Ok(modDate) => {
                                                                                                let datetime: DateTime<Local> = modDate.into();
                                                                                                let file_object = FileObject {
                                                                                                    id:id.clone(),
                                                                                                    name:file_name_str.clone(),
                                                                                                    isDir: true,
                                                                                                    childrenIds: Vec::new(),
                                                                                                    parentId: parentId.clone(),
                                                                                                    childrenCount: 0,
                                                                                                    size: metadata.file_size(),
                                                                                                    modDate: datetime.format("%Y/%m/%d %T").to_string(),
                                                                                                    thumbnailUrl: String::new(),
                                                                                                };
                                                                                                fsmap.insert(id.clone(), file_object.clone());
                                                                                                match fsmap_depth_map.get(&(entry.depth())) {
                                                                                                    Some(vector) => {
                                                                                                        let mut vector_new = vector.clone();
                                                                                                        vector_new.push(file_object.clone());
                                                                                                        fsmap_depth_map.insert(entry.depth(), vector_new);
                                                                                                    }
                                                                                                    None => {
                                                                                                        let mut vector_new = Vec::new();
                                                                                                        vector_new.push(file_object.clone());
                                                                                                        fsmap_depth_map.insert(entry.depth(), vector_new);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(e) => {
                                                                                                return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind().to_string()))));
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind().to_string()))));
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        let sth = json!({
                                                                            "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                            "reasons":e.to_string(),
                                                                        }); // 创造serde_json变量（类型叫Value）
                                                                        let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                                        return Ok(sth_warp);
                                                                    }
                                                                }
                                                            }
                                                            let sth = json!({
                                                                "status":config::API_STATUS_SUCCESS,
                                                                "data": {
                                                                    "rootFolderId": digest(format!("{}{}", config::DIR_STATIC, config::DIR_FTP)),
                                                                    "fileMap": fsmap,
                                                                }
                                                            }); // 创造serde_json变量（类型叫Value）
                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                            return Ok(sth_warp);
                                                        } else {
                                                            let sth = json!({
                                                                "status":config::API_STATUS_FAILURE_WITH_REASONS,
                                                                "reasons":"仅山林寺成员可查看"
                                                            }); // 创造serde_json变量（类型叫Value）
                                                            let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
                                                            return Ok(sth_warp);
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
                                                    return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e))));
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
                                    return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind.to_string()))));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind.to_string()))));
                        }
                    }
                }
                Err(e) => {
                    return Err(warp::reject::custom(FailedToGetFsMap(Box::new(e.kind.to_string()))));
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