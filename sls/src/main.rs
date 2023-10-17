use std::env;
use std::net::{IpAddr, SocketAddr};

use warp::Filter;

use get_comment_with_id::fun_get_comment_with_id;
use get_comments::{fun_get_comments, GetCommentsConfig};
use get_comments_of_comments::{fun_get_comments_of_comments, GetCommentsOfCommentsConfig};
use get_favorite_posts_with_student_id::{fun_get_favorite_posts_with_student_id, GetFavoritePostsWithStudentIdConfig};
use get_post_with_id::fun_get_post_with_id;
use get_posts::{fun_get_posts, GetPostsConfig};
use get_posts_with_student_id::{fun_get_posts_with_student_id, GetPostsWithStudentIdConfig};
use get_sls_members::fun_get_sls_members;
use get_user_profile::fun_get_user_profile;
use get_user_profile_with_student_id::fun_get_user_profile_with_student_id;
use read_image_files_in_folder::fun_read_image_files_in_folder;
use submit_an_action::{Action, fun_submit_an_action};
use submit_files::fun_submit_files;
use submit_login_info::{fun_submit_login_info, LoginInfo};
use submit_new_comment::{fun_submit_new_comment, NewComment};
use submit_new_post::{fun_submit_new_post, NewPost};
use submit_signup_info::{fun_submit_signup_info, SignUpInfo};
use get_sls_member_profile::fun_get_sls_member_profile;

mod config;
mod get_sls_members;
mod read_image_files_in_folder;
mod submit_signup_info;
mod submit_login_info;
mod token;
mod get_user_profile;
mod submit_new_post;
mod submit_files;
mod get_posts;
mod get_user_profile_with_student_id;
mod get_post_with_id;
mod get_comments;
mod submit_new_comment;
mod get_comment_with_id;
mod get_comments_of_comments;
mod submit_an_action;
mod get_posts_with_student_id;
mod get_favorite_posts_with_student_id;
mod get_sls_member_profile;

#[tokio::main]
async fn main() {
    // 设置日志
    env::set_var("RUST_APP_LOG", "debug");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");
    let info_log = warp::log("info_log");

    // 设置cors
    let origin: &str = &format!("http://{}:{}", IpAddr::from(config::FRONT_URL), config::FRONT_PORT);
    let cors = warp::cors()
        .allow_origin(origin)
        .allow_credentials(true)
        .allow_headers(vec!["Cookie", "Access-Control-Allow-Credentials", "Access-Control-Allow-Origin", "Content-Type"])
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

    // API3：向数据库写入新注册的账号
    // url:./submit_signup_info
    // 参数：json
    // 返回：json
    let submit_signup_info = warp::post()
        .and(warp::path("submit_signup_info"))
        .and(warp::path::end())
        .and(warp::body::json::<SignUpInfo>())
        .and_then(fun_submit_signup_info);

    // API4：检查数据库是否可登录账号
    // url:./submit_login_info
    // 参数：json
    // 返回：json
    let submit_login_info = warp::post()
        .and(warp::path("submit_login_info"))
        .and(warp::path::end())
        .and(warp::body::json::<LoginInfo>())
        .and_then(fun_submit_login_info);

    // API5：根据Cookie中的token，获取用户资料
    // url:./get_user_profile
    // 参数：无
    // 返回：json
    let get_user_profile = warp::get()
        .and(warp::path("get_user_profile"))
        .and(warp::path::end())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_get_user_profile);

    // API6：向数据库写入新发的帖子
    // url:./submit_new_post/编号
    // 参数：json
    // 返回：json
    let submit_new_post = warp::post()
        .and(warp::path("submit_new_post"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<NewPost>())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_new_post);

    // API7：存储文件
    // url:./submit_files/编号
    // 参数：formdata
    // 返回：json
    let submit_files = warp::post()
        .and(warp::path("submit_files"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::multipart::form().max_length(None))
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_files);

    // API8：读取帖子
    // url:./get_posts
    // 参数：json
    // 返回：json
    let get_posts = warp::post()
        .and(warp::path("get_posts"))
        .and(warp::path::end())
        .and(warp::body::json::<GetPostsConfig>())
        .and_then(fun_get_posts);

    // API9：根据账号获取用户资料
    // url:./get_user_profile_with_student_id/编号
    // 参数：无
    // 返回：json
    let get_user_profile_with_student_id = warp::get()
        .and(warp::path("get_user_profile_with_student_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(fun_get_user_profile_with_student_id);

    // API10：根据帖子编号获取帖子
    // url:./get_post_with_id/编号
    // 参数：无
    // 返回：json
    let get_post_with_id = warp::get()
        .and(warp::path("get_post_with_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(fun_get_post_with_id);

    // API11：读取评论
    // url:./get_comments/编号
    // 参数：json
    // 返回：json
    let get_comments = warp::post()
        .and(warp::path("get_comments"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<GetCommentsConfig>())
        .and_then(fun_get_comments);

    // API12：向数据库写入给的帖子的评论
    // url:./submit_new_comment/编号
    // 参数：json
    // 返回：json
    let submit_new_comment = warp::post()
        .and(warp::path("submit_new_comment"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<NewComment>())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_new_comment);

    // API13：根据评论编号获取评论
    // url:./get_comment_with_id/编号
    // 参数：无
    // 返回：json
    let get_comment_with_id = warp::get()
        .and(warp::path("get_comment_with_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(fun_get_comment_with_id);

    // API14：读取评论的评论
    // url:./get_comments_of_comments/编号
    // 参数：json
    // 返回：json
    let get_comments_of_comments = warp::post()
        .and(warp::path("get_comments_of_comments"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<GetCommentsOfCommentsConfig>())
        .and_then(fun_get_comments_of_comments);

    // API15：向数据库写入给的帖子的动作
    // url:./submit_a_action
    // 参数：json
    // 返回：json
    let submit_an_action = warp::post()
        .and(warp::path("submit_an_action"))
        .and(warp::path::end())
        .and(warp::body::json::<Action>())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_an_action);

    // API16：读取指定用户所发的帖子
    // url:./get_posts_with_student_id/编号
    // 参数：json
    // 返回：json
    let get_posts_with_student_id = warp::post()
        .and(warp::path("get_posts_with_student_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<GetPostsWithStudentIdConfig>())
        .and_then(fun_get_posts_with_student_id);

    // API17：读取指定用户所收藏的帖子
    // url:./get_favorite_posts_with_student_id/编号
    // 参数：json
    // 返回：json
    let get_favorite_posts_with_student_id = warp::post()
        .and(warp::path("get_favorite_posts_with_student_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<GetFavoritePostsWithStudentIdConfig>())
        .and_then(fun_get_favorite_posts_with_student_id);

    // API18：根据Cookie中的token，获取山林寺成员资料
    // url:./get_sls_member_profile
    // 参数：无
    // 返回：json
    let get_sls_member_profile = warp::get()
        .and(warp::path("get_sls_member_profile"))
        .and(warp::path::end())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_get_sls_member_profile);

    // 合并路由
    let dir_static = warp::fs::dir(config::DIR_STATIC);
    let route = dir_static
        .or(get_sls_members)
        .or(read_image_files_in_folder)
        .or(submit_signup_info)
        .or(submit_login_info)
        .or(get_user_profile)
        .or(submit_new_post)
        .or(submit_files)
        .or(get_posts)
        .or(get_user_profile_with_student_id)
        .or(get_post_with_id)
        .or(get_comments)
        .or(submit_new_comment)
        .or(get_comment_with_id)
        .or(get_comments_of_comments)
        .or(submit_an_action)
        .or(get_posts_with_student_id)
        .or(get_favorite_posts_with_student_id)
        .or(get_sls_member_profile)
        .with(info_log)
        .with(cors);

    // 使路由链接到自身ip地址
    warp::serve(route)
        .run(SocketAddr::new(IpAddr::from(config::SERVER_URL), config::SERVER_PORT))
        .await; // 阻塞运行
}