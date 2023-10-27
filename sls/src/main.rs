use std::env;

use warp::Filter;

use get_admin_profile::fun_get_admin_profile;
use get_comment_with_id::fun_get_comment_with_id;
use get_comments::{fun_get_comments, GetCommentsConfig};
use get_comments_of_comments::{fun_get_comments_of_comments, GetCommentsOfCommentsConfig};
use get_favorite_posts_with_student_id::{fun_get_favorite_posts_with_student_id, GetFavoritePostsWithStudentIdConfig};
use get_post_with_id::fun_get_post_with_id;
use get_posts::{fun_get_posts, GetPostsConfig};
use get_posts_with_student_id::{fun_get_posts_with_student_id, GetPostsWithStudentIdConfig};
use get_sls_member_profile::fun_get_sls_member_profile;
use get_sls_member_profile_with_student_id::fun_get_sls_member_profile_with_student_id;
use get_sls_members::fun_get_sls_members;
use get_text::fun_get_text;
use get_user_profile::fun_get_user_profile;
use get_user_profile_with_student_id::fun_get_user_profile_with_student_id;
use read_image_files_in_folder::fun_read_image_files_in_folder;
use submit_admin_login_info::fun_submit_admin_login_info;
use submit_an_action::{Action, fun_submit_an_action};
use submit_comment_removing::{CommentRemoving, fun_submit_comment_removing};
use submit_files::fun_submit_files;
use submit_images::fun_submit_images;
use submit_login_info::{fun_submit_login_info, LoginInfo};
use submit_new_comment::{fun_submit_new_comment, NewComment};
use submit_new_post::{fun_submit_new_post, NewPost};
use submit_new_sls_member::{fun_submit_new_sls_member, NewSlsMember};
use submit_new_text::{fun_submit_new_text, NewText};
use submit_photo_removing::{fun_submit_photo_removing, PhotoRemoving};
use submit_post_removing::{fun_submit_post_removing, PostRemoving};
use submit_signup_info::{fun_submit_signup_info, SignUpInfo};
use submit_sls_member_image::fun_submit_sls_member_image;
use submit_sls_member_moving::{fun_submit_sls_member_moving, SlsMemberMoving};
use submit_sls_member_profile_update::{fun_submit_sls_member_profile_update, SlsMemberProfileUpdate};
use submit_sls_member_removing::{fun_submit_sls_member_removing, SlsMemberRemoving};
use get_fsmap::fun_get_fsmap;

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
mod submit_sls_member_profile_update;
mod get_sls_member_profile_with_student_id;
mod submit_sls_member_image;
mod submit_admin_login_info;
mod submit_new_sls_member;
mod get_admin_profile;
mod submit_sls_member_removing;
mod submit_post_removing;
mod submit_comment_removing;
mod submit_sls_member_moving;
mod submit_photo_removing;
mod submit_images;
mod get_text;
mod submit_new_text;
mod get_fsmap;

#[tokio::main]
async fn main() {
    // 设置日志
    env::set_var("RUST_APP_LOG", "debug");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");
    let info_log = warp::log("info_log");

    // 设置cors
    let origin: &str = &format!("http://{}:{}", config::FRONT_URL, config::FRONT_PORT);
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
        .and(warp::path::param::<String>())
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

    // API19：向数据库更新山林寺认证信息
    // url:./submit_sls_member_profile_update
    // 参数：json
    // 返回：json
    let submit_sls_member_profile_update = warp::post()
        .and(warp::path("submit_sls_member_profile_update"))
        .and(warp::path::end())
        .and(warp::body::json::<SlsMemberProfileUpdate>())
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_sls_member_profile_update);

    // API20：根据账号获取山林寺成员资料
    // url:./get_sls_member_profile_with_student_id/编号
    // 参数：无
    // 返回：json
    let get_sls_member_profile_with_student_id = warp::get()
        .and(warp::path("get_sls_member_profile_with_student_id"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(fun_get_sls_member_profile_with_student_id);

    // API21：存储山林寺成员照片
    // url:./submit_sls_member_image
    // 参数：formdata
    // 返回：json
    let submit_sls_member_image = warp::post()
        .and(warp::path("submit_sls_member_image"))
        .and(warp::path::end())
        .and(warp::multipart::form().max_length(None))
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_submit_sls_member_image);

    // API22：检查数据库是否可登录管理员账号
    // url:./submit_admin_login_info
    // 参数：json
    // 返回：json
    let submit_admin_login_info = warp::post()
        .and(warp::path("submit_admin_login_info"))
        .and(warp::path::end())
        .and(warp::body::json::<LoginInfo>())
        .and_then(fun_submit_admin_login_info);

    // API23：向数据库写入新的山林寺成员
    // url:./submit_new_sls_member/类型
    // 参数：json
    // 返回：json
    let submit_new_sls_member = warp::post()
        .and(warp::path("submit_new_sls_member"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<NewSlsMember>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_new_sls_member);

    // API24：根据Cookie中的token，获取管理员资料
    // url:./get_admin_profile
    // 参数：无
    // 返回：json
    let get_admin_profile = warp::get()
        .and(warp::path("get_admin_profile"))
        .and(warp::path::end())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_get_admin_profile);

    // API25：数据库删除指定的山林寺成员
    // url:./submit_sls_member_removing/类型
    // 参数：json
    // 返回：json
    let submit_sls_member_removing = warp::post()
        .and(warp::path("submit_sls_member_removing"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<SlsMemberRemoving>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_sls_member_removing);

    // API26：数据库删除指定的帖子
    // url:./submit_post_removing
    // 参数：json
    // 返回：json
    let submit_post_removing = warp::post()
        .and(warp::path("submit_post_removing"))
        .and(warp::path::end())
        .and(warp::body::json::<PostRemoving>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_post_removing);

    // API27：数据库删除指定的评论
    // url:./submit_comment_removing
    // 参数：json
    // 返回：json
    let submit_comment_removing = warp::post()
        .and(warp::path("submit_comment_removing"))
        .and(warp::path::end())
        .and(warp::body::json::<CommentRemoving>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_comment_removing);

    // API28：数据库转移指定的山林寺成员
    // url:./submit_sls_member_moving/类型
    // 参数：json
    // 返回：json
    let submit_sls_member_moving = warp::post()
        .and(warp::path("submit_sls_member_moving"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<SlsMemberMoving>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_sls_member_moving);

    // API29：数据库删除指定的照片墙照片
    // url:./submit_photo_removing
    // 参数：json
    // 返回：json
    let submit_photo_removing = warp::post()
        .and(warp::path("submit_photo_removing"))
        .and(warp::path::end())
        .and(warp::body::json::<PhotoRemoving>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_photo_removing);

    // API30：存储照片墙照片
    // url:./submit_photo_wall_images
    // 参数：formdata
    // 返回：json
    let submit_images = warp::post()
        .and(warp::path("submit_images"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::multipart::form().max_length(None))
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_images);

    // API31：读取文本
    // url:./get_texts/类型
    // 参数：无
    // 返回：json
    let get_text = warp::get() // 使用get方式
        .and(warp::path("get_text")) // url元素
        .and(warp::path::param::<String>())
        .and(warp::path::end()) // url结束
        .and_then(fun_get_text); // 响应方式

    // API32：向数据库写入新的文本
    // url:./submit_new_text/类型
    // 参数：json
    // 返回：json
    let submit_new_text = warp::post()
        .and(warp::path("submit_new_text"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json::<NewText>())
        .and(warp::filters::cookie::optional("admin_token"))
        .and_then(fun_submit_new_text);

    // API33：读取文件树
    // url:./get_fsmap
    // 参数：无
    // 返回：json
    let get_fsmap = warp::get() // 使用get方式
        .and(warp::path("get_fsmap")) // url元素
        .and(warp::path::end()) // url结束
        .and(warp::filters::cookie::optional("token"))
        .and_then(fun_get_fsmap); // 响应方式

    // 合并路由
    let dir_static = warp::fs::dir(config::DIR_STATIC);
    let route = dir_static
        .or(get_sls_members)
        .or(read_image_files_in_folder)
        .or(submit_signup_info)
        .or(submit_login_info)
        .boxed()
        .or(get_user_profile)
        .or(submit_new_post)
        .or(submit_files)
        .or(get_posts)
        .or(get_user_profile_with_student_id)
        .or(get_post_with_id)
        .boxed()
        .or(get_comments)
        .or(submit_new_comment)
        .or(get_comment_with_id)
        .or(get_comments_of_comments)
        .or(submit_an_action)
        .or(get_posts_with_student_id)
        .or(get_favorite_posts_with_student_id)
        .boxed()
        .or(get_sls_member_profile)
        .or(submit_sls_member_profile_update)
        .or(get_sls_member_profile_with_student_id)
        .or(submit_sls_member_image)
        .or(submit_admin_login_info)
        .or(submit_new_sls_member)
        .or(get_admin_profile)
        .or(submit_sls_member_removing)
        .or(submit_post_removing)
        .boxed()
        .or(submit_comment_removing)
        .or(submit_sls_member_moving)
        .or(submit_photo_removing)
        .or(submit_images)
        .or(get_text)
        .or(submit_new_text)
        .or(get_fsmap)
        .with(info_log)
        .with(cors);
    //调试时不加boxed会因为or太多而溢出，release时可能可以去掉

    // 使路由链接到自身ip地址
    warp::serve(route)
        .run((config::SELF_URL, config::SELF_PORT))
        .await; // 阻塞运行
}