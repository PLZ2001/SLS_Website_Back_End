use serde_json::json;
use walkdir::WalkDir;

use crate::config;

#[derive(Debug)]
pub struct FailedToReadImageFilesInFolder(Box<String>);

impl warp::reject::Reject for FailedToReadImageFilesInFolder {}

pub async fn fun_read_image_files_in_folder(folder_category: String) -> Result<warp::reply::Json, warp::Rejection> {
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
    let mut image_files = Vec::new();
    for entry in WalkDir::new(format!("{}{}", config::DIR_STATIC, folder))
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                match extension.to_str() {
                    Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp") => {
                        if let Some(file_name) = entry.path().file_stem() {
                            let file_name_str = file_name.to_string_lossy().to_string();
                            let file_path_str = entry.path().to_string_lossy().to_string().replace("\\", "/");
                            image_files.push(json!({
                                "image":format!("http://{}:{}/{}", config::SERVER_URL, config::SERVER_PORT, &file_path_str[config::DIR_STATIC.len()..]),
                                "title":file_name_str,
                                "file_path":file_path_str
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let sth = json!({
        "status":config::API_STATUS_SUCCESS,
        "data":image_files
    }); // 创造serde_json变量（类型叫Value）
    let sth_warp = warp::reply::json(&sth); // 转换为warp的json格式
    return Ok(sth_warp);
}