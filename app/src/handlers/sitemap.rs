use crate::Config;
use crate::get_cache_urls;
use crate::SitemapTemplate;
// use crate::AppState;
use crate::RequestState;
use crate::functions::sql::PgsqlService;
use axum::{
    self,
    http::StatusCode,
    body::Body,
    extract::Request,
    response::{Html, IntoResponse, Redirect, Response},
    Extension
};
use askama::Template;

// use sqlx::PgPool;
use std::sync::{Arc, RwLock};

// // 生成 SEO404 页面
// let (url, title) = my_func
//     .get_push_link(linecache, "doc/404_links.txt".to_string())
//     .await;
// let template = Seo404Template {
//     title: title,
//     url: url,
// };
// return Ok(template
//     .render()
//     .map(Html)
//     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
//     .into_response());


pub async fn sitemap(
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let req_state = {
        req.extensions().get::<RequestState>().unwrap().clone()
    };

    let domain_info = &req_state.domain_info;
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");

    let body = match get_cache_urls(&pgsql, &table_name, "缓存").await {
        Some(urls) => {
            let mut new_urls = Vec::new();
            for url in urls.iter() {
                new_urls.push(format!("{}:{}", &req_state.scheme, url));
            }
            let template = SitemapTemplate {
                base_url: format!("{}://{}", req_state.scheme, domain_info["full_domain"]),
                urls: new_urls,
            };
            template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        },
        None => {
            "".to_string()
        }
    };
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/xml; charset=utf-8")
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap())
}


pub async fn sitemap_txt(
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let req_state = {
        req.extensions().get::<RequestState>().unwrap().clone()
    };

    let domain_info = &req_state.domain_info;
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");

    let body = match get_cache_urls(&pgsql, &table_name, "缓存").await {
        Some(urls) => {
            let mut urls_text = "".to_string();
            for url in urls.iter() {
                urls_text.push_str(&format!("{}:{}\n", &req_state.scheme, url));
            }
            urls_text.trim_end().to_string()
        },
        None => {
            "".to_string()
        }
    };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap())
}