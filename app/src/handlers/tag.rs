// use crate::handlers::website_main::fetch_or_create_config;
use crate::{functions::func::MyFunc, AppState, Config, PgsqlService, RequestState};
use std::path::Path;
// use crate::functions::sql::PgsqlService;
use axum::{
    self,
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use linecache::AsyncLineCache;
use minio_rsc::Minio;
use tokio::fs::read_to_string;
// use sqlx::PgPool;
use std::sync::{Arc, RwLock};

pub async fn tag_html(
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Extension(state): Extension<AppState>,
    Extension(config): Extension<Arc<RwLock<Config>>>,
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let req_state = req.extensions().get::<RequestState>().unwrap();
    // 获取 config_dict
    let config_dict = config.read().unwrap().clone();
    // 异步读取 HTML 文件
    let html_content = match read_to_string(Path::new("_/tag.html")).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read tag.html: {}", e);
            "".to_string()
        }
    };
    let domain_info = &req_state.domain_info;
    let config_path = format!(
        "{}/{}.toml",
        domain_info["root_domain"], domain_info["full_domain"]
    );

    // 调用单独的函数处理获取或生成配置文件的逻辑
    let webconfig = my_func.fetch_or_create_config(
        domain_info["subdomain"].as_str() == "www",
        &config_dict,
        &pgsql,
        &config_path,
        &domain_info["full_domain"],
    )
    .await?;

    let new_html = my_func
        .tag_parse(
            &config_dict,
            &webconfig,
            &req_state,
            &pgsql,
            &linecache,
            html_content,
        )
        .await;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(new_html))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap())
}
