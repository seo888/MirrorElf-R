use crate::Config;
// use crate::functions::sql::PgsqlService;
use axum::{
    self,
    http::StatusCode,
    body::Body,
    extract::Request,
    response::Response,
    Extension
};
// use sqlx::PgPool;
use std::sync::{Arc, RwLock};


pub async fn verify_adjs(
    // Extension(config): Extension<Arc<RwLock<Config>>>,
    // Extension(pgsql): Extension<Arc<PgsqlService>>,
    req: Request,
) -> Result<Response, StatusCode> {


    let js_text = r#"
var ss = '/_static/ad.html';
document.write(`
    <meta id="viewport" name="viewport" content="user-scalable=no,width=device-width, initial-scale=1.0" />
    <style>
        html, body { width: 100%; height: 100%; overflow: hidden; clear: both; }
        body > *, .container { opacity: 0; }
        #divs { opacity: 1; }
    </style>
    <div style="position:absolute; top:0; left:0; width:100%; height:100%; z-index:2147483647;" id="divs">
        <iframe src="${ss}" frameborder="0" style="border:0; width:100%; height:100%; max-height:4000px;"></iframe>
    </div>
`);
    "#;
    // let body = format!("url_path: {} app_name: {}", url_path, config_read.app_name);
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/javascript")
        .body(Body::from(js_text))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap())
}

pub async fn verify_adhtml(
    // Extension(config): Extension<Arc<RwLock<Config>>>,
    // Extension(pgsql): Extension<Arc<PgsqlService>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let html_content = r#"<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>程序已到期</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            background-color: #f0f0f0;
            font-family: Arial, sans-serif;
        }
        .content {
            text-align: center;
            font-size: 20px;
            color: #333;
            padding: 20px;
            background-color: #ffffff;
            border-radius: 8px;
            box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
        }
        .logo {
            max-width: 80%;
            height: auto;
            margin-bottom: 20px;
        }
        a {
            color: #007BFF;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="content">
        <img src="/_/admin/logo.png" alt="Logo" class="logo">
        <p>镜像精灵 Mirror-Elf ®</p>
        <p>程序已到期，请进入 <a href="/_/admin">网站后台</a> -> 设置 -> 操作续费</p>
        <p>© 2020-2025 <a href="https://t.me/MirrorElf" target="_blank">https://t.me/MirrorElf</a></p> 
    </div>
</body>
</html>"#;
    // let body = format!("url_path: {} app_name: {}", url_path, config_read.app_name);
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html_content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap())
}