mod config;
mod get_sls_members;
mod read_image_files_in_folder;

use get_sls_members::fun_get_sls_members;
use read_image_files_in_folder::fun_read_image_files_in_folder;

use std::env;
use std::net::{IpAddr, SocketAddr};
use warp::Filter;

#[tokio::main]
async fn main() {
    // 设置日志
    env::set_var("RUST_APP_LOG", "debug");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");
    let info_log = warp::log("info_log");

    // 设置cors
    let cors = warp::cors()
        .allow_any_origin()
        .allow_credentials(true)
        .allow_headers(vec!["access-control-allow-origin", "content-type"])
        .allow_methods(vec!["POST", "GET", "PUT", "DELETE"]);

    // 设置路由

    // API1：读取山林寺成员名单
    // url:./get_sls_members
    // 参数：无
    // 返回：json
    let get_sls_members = warp::get() // 使用get方式
        .and(warp::path("get_sls_members")) // url元素
        .and(warp::path::end()) // url结束
        .and_then(fun_get_sls_members); // 响应方式

    // API2：读取照片墙的照片名单
    // url:./read_image_files_in_folder
    // 参数：无
    // 返回：json
    let read_image_files_in_folder = warp::get() // 使用get方式
        .and(warp::path("read_image_files_in_folder")) // url元素
        .and(warp::path::end()) // url结束
        .and_then(fun_read_image_files_in_folder); // 响应方式

    // 合并路由
    let dir_static = warp::fs::dir(config::DIR_STATIC);
    let route = dir_static.or(get_sls_members).or(read_image_files_in_folder).with(info_log).with(cors);

    // 使路由链接到自身ip地址
    warp::serve(route)
        .run(SocketAddr::new(IpAddr::from(config::SERVER_URL), config::SERVER_PORT))
        .await; // 阻塞运行
}