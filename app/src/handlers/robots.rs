use crate::get_cache_machine_id;
use crate::{load_config, Config};
use crate::my_const::CONFIG_FILE_PATH;
use crate::functions::verify::Verify;
use axum::{
    body::Body,
    extract::{Json, Query, Request},
    http::{header, header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use std::sync::{Arc, RwLock};

pub async fn robots(
    Extension(verify): Extension<Arc<Verify>>,
    Extension(config): Extension<Arc<RwLock<Config>>>,
    req: Request,
) -> Result<Response<String>, StatusCode>  {
    // 获取 config
    let config_dict = config.read().unwrap().clone();

    if config_dict.website_settings.auto_https_certificate {

        let verify_info = match verify
            .decrypt_data(
                &config_dict.program_info.authorization_code,
                get_cache_machine_id().await,
            )
            .await
        {
            Ok(r_info) => {
                r_info
            }
            Err(r_info) => {
                // 处理错误情况
                // 将config 授权状态auto_https_certificate 改为 false
                // 将 Config 序列化为 YAML 格式
                let mut yaml_data = serde_yaml::to_string(&config_dict).map_err(|e| {
                    eprintln!("Failed to serialize config to YAML: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                
                // 将auto_https_certificate: true 改为 auto_https_certificate: false
                yaml_data = yaml_data.replace("auto_https_certificate: true","auto_https_certificate: false");
            
                // 异步保存文件内容
                match tokio::fs::write(CONFIG_FILE_PATH, yaml_data).await {
                    Ok(_) => {
                        println!("{} File saved successfully.", CONFIG_FILE_PATH);
                    }
                    Err(e) => {
                        // 处理文件保存错误
                        println!("{} Failed to save file: {}", CONFIG_FILE_PATH, e);
                    }
                }
                r_info
            }
        };
    }

    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let robots_text = format!(
        "User-agent: *\n\
        Disallow: /_api\n\
        Sitemap: {scheme}://{host}/sitemap.xml\n\
        Sitemap: {scheme}://{host}/sitemap.txt",
        scheme = req.uri().scheme_str().unwrap_or("http"),
        host = host
    );

    Ok(Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(robots_text)
        .unwrap())
}
