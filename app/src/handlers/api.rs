use crate::domain_info_from_domain;
use crate::functions::func::MyFunc;
use crate::Claims;
// use crate::functions::minio::MinioClient;
use crate::functions::verify::Verify;
use crate::get_cache_machine_id;
use crate::get_cache_version_text;
use crate::my_const::{CONFIG_FILE_PATH, REPALCE_CONTENT, SECRET, VERSION};
use crate::AppState;
use crate::AsyncLineCache;
use crate::IncludeInfo;
use crate::MuluConfig;
use crate::ReplaceRules;
use crate::TargetReplaceRules;
use crate::WebsiteConf;
use crate::WebsiteInfo;
use crate::{load_config, Config, PgsqlService};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
// use std::collections::HashMap;
// use rand::Rng;
use std::fmt;
use std::sync::{Arc, RwLock};
// use anyhow::{Error, Result};
use axum::{
    body::Body,
    // extract::State,
    // http::StatusCode,
    extract::{Json, Query, Request},
    http::{header, header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{sse::Event, sse::KeepAlive, IntoResponse, Response, Sse},
    Extension,
};
// use std::time::Duration;
// use tokio::sync::broadcast;
// use tokio::time::{self, sleep};
// use tokio_stream::StreamExt;
// use bytes::Bytes;
use sqlx::{postgres::PgRow, PgPool, Row};
// use futures::StreamExt;
use minio_rsc::{client::KeyArgs, client::ListObjectsArgs, Minio};
// use rand_user_agent::UserAgent;
// use reqwest::Client;
// use async_stream::stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;
// use tokio::io::{self, AsyncBufReadExt, BufReader};
// use futures::stream::iter;
use futures::stream::{unfold, Stream};
// use tokio_stream::wrappers::LinesStream;
// use tokio_stream::Stream;
// use tokio_stream::StreamExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
// use linecache::AsyncLineCache;
// use serde_json::Value;
// use tokio::sync::mpsc;
// use tokio::time::Instant;
// use tokio_stream::wrappers::ReceiverStream;
// use tokio_util::io::StreamReader;
// use tracing::{error, info};
// const REPALCE_CONTENT: &str = "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'";

// async fn validate_login(account: &str, password: &str) -> bool {
//     // 示例验证逻辑：这里只是简单地检查账号和密码是否匹配

//     if let (Some(login_account), Some(login_password)) = (account, password) {
//         // 计算密码的 MD5 哈希值
//         let mut hasher = Md5::new();
//         hasher.update(password.as_bytes());
//         let md5_hashed_password = format!("{:x}", hasher.finalize());

//         // 验证账号和密码
//         return account == login_account && md5_hashed_password == login_password;
//     }
//     false
// }

pub async fn sql_test(
    // Extension(config): Extension<Arc<RwLock<Config>>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    // req: Request,
) -> Result<Response, StatusCode> {
    // let name = config.read().unwrap().program_info.program_name.clone();
    let version = match pgsql.get_db_version().await {
        Ok(version) => {
            println!("数据库版本: {}", version);
            version
        }
        Err(status) => {
            println!("获取版本失败: {}", status);
            return Err(status);
        }
    };
    return Ok(Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(version))
        .unwrap());
}

#[derive(Debug, Deserialize)]
pub struct LogintData {
    account: Option<String>,
    password: Option<String>,
}

pub async fn login(
    Extension(config): Extension<Arc<RwLock<Config>>>,
    Json(json_data): Json<LogintData>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let account = json_data.account;
    let password = json_data.password;

    // 简单的账号和密码格式验证
    if account.is_none() || password.is_none() {
        let json_result = json!({"msg": "账号和密码不能为空","status": -1});
        return Ok(Json(json_result));
    }

    let account = account.unwrap();
    let md5_password: String = password.unwrap();

    // 获取 config
    let config_dict = config.read().unwrap().clone();

    // 计算密码的 MD5 哈希值
    let md5_hashed_password = format!(
        "{:x}",
        md5::compute(config_dict.program_info.login_password.as_bytes())
    );
    println!("account:{} md5_password:{}", account, md5_password);
    println!(
        "dict_account:{} dict_password:{} password:{}",
        config_dict.program_info.login_account,
        md5_hashed_password,
        config_dict.program_info.login_password
    );

    // 验证账号和密码
    if account == config_dict.program_info.login_account && md5_password == md5_hashed_password {
        // 生成 JWT token
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token 2小时后过期

        let claims = Claims {
            sub: account,
            exp: exp.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap();
        let json_result = json!({"msg": "登录成功","status": 0,"token": Some(token)});
        return Ok(Json(json_result));
    } else {
        let json_result = json!({"msg": "账号或密码错误","status": -1});
        return Ok(Json(json_result));
    }
}

#[derive(Deserialize)]
pub struct VersionParams {
    mode: i32,
}

pub async fn version(
    Query(params): Query<VersionParams>, // 提取查询参数
    Extension(my_func): Extension<Arc<MyFunc>>,
) -> Result<Response, StatusCode> {
    let mode = params.mode;
    let mut version_text = "".to_string();
    if mode > 0 {
        version_text = match get_cache_version_text(my_func, VERSION).await {
            Ok(text) => text,
            Err(e) => {
                println!("Failed to fetch URL: {}", e);
                return Err(e);
            }
        };
    }

    let title = if version_text.contains("🚀") {
        version_text.to_string() // 或 version_text.clone()，取决于是否需要所有权
    } else {
        "当前已是最新版本".to_string()
    };

    if version_text.contains("🚀") {
        version_text = format!("{}🚀", version_text.split("🚀").next().unwrap_or(""));
    } else {
        version_text = format!("{} ", version_text);
    }

    let machine_id = match get_cache_machine_id().await {
        Some(id) => id,
        None => {
            // 处理错误情况
            println!("Failed to get machine ID");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let json_result = json!({"data":{"title":title,"version":version_text,"machine_id":machine_id},"msg": "获取版本号 成功", "status": 0});
    return Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap());
}

pub async fn program_name(
    Extension(config): Extension<Arc<RwLock<Config>>>,
    // req: Request,
) -> Result<Response, StatusCode> {
    let name = config.read().unwrap().program_info.program_name.clone();
    return Ok(Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(name))
        .unwrap());
}

// #[derive(Deserialize)]
// pub struct LineParams {
//     path: Option<String>,
// }
// pub async fn line(
//     Query(params): Query<LineParams>,                     // 提取查询参数
//     Extension(linecache): Extension<Arc<AsyncLineCache>>,
// ) -> Result<Response, StatusCode> {
//     let file_path = params.path.unwrap_or("".to_string());
//     let line = linecache.random_line(&file_path).await.unwrap();
//     let version_text = "0.4.0";
//     let json_result = json!({"data":{"random":line},"msg": "测试 成功", "status": 0});
//     return Ok(Response::builder()
//         .header("Content-Type", "application/json")
//         .body(Body::from(json_result.to_string()))
//         .unwrap());
// }

// 自定义错误类型
// #[derive(Debug)]
// struct LogError {
//     message: String,
// }

// // 实现 std::error::Error
// impl std::error::Error for LogError {}

// // 实现 fmt::Display
// impl fmt::Display for LogError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "LogError: {}", self.message)
//     }
// }

// 将 logs 函数包装为 axum 的 Handler
// pub async fn logs_handler() -> impl IntoResponse {
//     match logs().await {
//         Ok(response) => response,
//         Err(status) => (status, "Internal Server Error").into_response(),
//     }
// }

pub async fn logs(
    // State(state): State<Arc<AppState>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    // 从状态中获取广播接收器
    let mut rx = state.tx.subscribe();

    // 创建一个 SSE 流
    let stream = async_stream::stream! {
        while let Ok(log_entry) = rx.recv().await {
            yield Ok(Event::default().data(log_entry));
        }
    };

    // 返回 SSE 响应
    Sse::new(stream).keep_alive(KeepAlive::default())
}

// pub async fn logs(req: Request) -> Result<Response, StatusCode> {
//     // 获取当前日期
//     let today = Local::now().format("%Y-%m-%d").to_string();
//     let file_path = format!("log/app.log.{}", today);

//     // 打开日志文件
//     let file = match fs::File::open(&file_path).await {
//         Ok(file) => file,
//         Err(e) => {
//             eprintln!("Failed to open log file: {}", e);
//             return Err(StatusCode::NOT_FOUND);
//         }
//     };

//     // 创建 BufReader
//     let reader = BufReader::new(file);

//     // 创建一个异步流
//     // let stream = stream! {
//     //     let mut lines = reader.lines();
//     //     while let Some(result) = lines.next_line().await {
//     //         match result {
//     //             Ok(line) => yield Ok(line + "\n"),
//     //             Err(e) => {
//     //                 eprintln!("Failed to read line from log file: {}", e);
//     //                 break; // 或者继续读取下一行
//     //             }
//     //         }
//     //     }
//     // };
//     let stream = stream! {
//         let mut lines = reader.lines();
//         while let Some(result) = lines.next_line().await {
//             match result {
//                 Ok(Some(line)) => yield Ok(line + "\n"),
//                 Ok(None) => break, // 文件读取结束
//                 Err(e) => {
//                     eprintln!("Failed to read line from log file: {}", e);
//                     break; // 或者继续读取下一行
//                 }
//             }
//         }
//     };

//     // 构造 HTTP 响应
//     let response = Response::builder()
//         .status(StatusCode::OK)
//         .header("Content-Type", "text/plain; charset=utf-8")
//         .body(Body::from_stream(stream))
//         .unwrap();

//     Ok(response)
// }

// #[derive(Deserialize)]
// pub struct ConfigQueryParams {
//     file_path: String,
// }

pub async fn config_query(
    Extension(verify): Extension<Arc<Verify>>,
) -> Result<Response<Body>, StatusCode> {
    // 先加载配置
    let new_config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // let machine_id = match get_cache_machine_id().await {
    //     Some(id) => id,
    //     None => {
    //         // 处理错误情况
    //         println!("Failed to get machine ID");
    //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
    //     }
    // };

    let verify_success: bool;
    let verify_info = match verify
        .decrypt_data(
            &new_config.program_info.authorization_code,
            get_cache_machine_id().await,
        )
        .await
    {
        Ok(r_info) => {
            verify_success = true;
            r_info
        }
        Err(r_info) => {
            // 处理错误情况
            verify_success = false;
            r_info
        }
    };

    // 将 new_config 转换为 serde_json::Value
    let mut config_value: Value = serde_json::to_value(&new_config).unwrap();

    // 添加自定义字段
    config_value["authorization_info"] = json!(verify_info);

    // 如果验证失败，添加验证信息
    if !verify_success {
        config_value["WebsiteSettings"]["auto_https_certificate"] = json!(false);
        config_value["authorization_end_info"] = json!(verify_info);
    } else {
        config_value["authorization_end_info"] = json!("");
    }

    // 构造最终的 JSON 响应
    let json_result = json!({
        "data": config_value,
        "msg": "配置文件获取成功",
        "status": 0
    });

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

// pub async fn config_query(// Query(params): Query<ConfigQueryParams>, // 提取查询参数
//     // Extension(verify): Extension<Arc<Verify>>,
// ) -> Result<Response, StatusCode> {
//     // 构造文件路径
//     // let file_path = format!("doc/{}", params.file_path);
//     // let file_path = "config/config.yml";
//     // let config_data = fs::read_to_string("config/config.yml").await?;
//     // let config: Config = serde_yaml::from_str(&config_data)?;
//     match load_config().await {
//         Ok(new_config) => {
//             println!("{:?}", new_config);
//             println!("Config reloaded successfully.");
//             let json_result = json!({"data":new_config,"msg": "配置文件获取 成功", "status": 0});
//             Ok(Response::builder()
//                 .header("Content-Type", "application/json")
//                 .body(Body::from(json_result.to_string()))
//                 .unwrap())
//         }
//         Err(e) => {
//             // 处理文件读取错误
//             eprintln!("Failed to read file: {}", e);
//             Err(StatusCode::NOT_FOUND)
//         }
//     }
// }

pub async fn config_update(
    Extension(verify): Extension<Arc<Verify>>,
    Json(config_data): Json<Config>,
) -> Result<Response<String>, StatusCode> {
    // 将 Config 序列化为 YAML 格式
    let mut yaml_data = serde_yaml::to_string(&config_data).map_err(|e| {
        eprintln!("Failed to serialize config to YAML: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let verify_success: bool;
    let _verify_info = match verify
        .decrypt_data(
            &config_data.program_info.authorization_code,
            get_cache_machine_id().await,
        )
        .await
    {
        Ok(r_info) => {
            verify_success = true;
            r_info
        }
        Err(r_info) => {
            // 处理错误情况
            verify_success = false;
            r_info
        }
    };
    if !verify_success {
        // 将auto_https_certificate: true 改为 auto_https_certificate: false
        yaml_data = yaml_data.replace(
            "auto_https_certificate: true",
            "auto_https_certificate: false",
        );
    } else {
        yaml_data = yaml_data.replace(
            "auto_https_certificate: false",
            "auto_https_certificate: true",
        );
    }

    // 文件路径
    let file_path = CONFIG_FILE_PATH;

    // 获取文件的元数据
    let metadata = fs::metadata(file_path).await.map_err(|e| {
        eprintln!("Failed to get file metadata: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 获取文件的最后修改时间
    let modified_time = metadata.modified().map_err(|e| {
        eprintln!("Failed to get file modified time: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 获取当前时间
    let current_time = SystemTime::now();

    // 计算时间差
    let time_diff = current_time.duration_since(modified_time).map_err(|e| {
        eprintln!("Failed to calculate time difference: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 如果时间差小于 60 秒，则返回失败
    if time_diff.as_secs() < 5 {
        let json_result = json!({
            "data": config_data,
            "msg": "配置文件更新失败：操作过于频繁，请稍后再试",
            "status": -1
        });

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(json_result.to_string())
            .unwrap());
    }

    // 异步保存文件内容
    let json_result;
    match fs::write(file_path, yaml_data).await {
        Ok(_) => {
            println!("{} File saved successfully.", file_path);
            // 返回更新后的配置内容
            json_result = json!({
                "data": config_data,
                "msg": "配置文件更新成功",
                "status": 0
            });
        }
        Err(e) => {
            // 处理文件保存错误
            println!("{} Failed to save file: {}", file_path, e);
            // 返回更新后的配置内容
            json_result = json!({
                "data": config_data,
                "msg": format!("配置文件更新失败 {}", e),
                "status": -1
            });
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json_result.to_string())
        .unwrap())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FilePutData {
    content: String,
    filepath: String,
}
pub async fn file_update(Json(data): Json<FilePutData>) -> Result<Response<String>, StatusCode> {
    let json_result;
    // 异步保存文件内容
    match fs::write(data.filepath, data.content).await {
        Ok(_) => {
            json_result = json!({
                "msg": format!("文件保存 成功"),
                "status": 0
            });
        }
        Err(e) => {
            // 处理文件保存错误
            json_result = json!({
                "msg": format!("文件保存 失败 {}", e),
                "status": -1
            });
        }
    }
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json_result.to_string())
        .unwrap())
}

#[derive(Deserialize)]
pub struct FileQueryParams {
    path: Option<String>,
}
pub async fn file_query(
    Query(params): Query<FileQueryParams>, // 提取查询参数
) -> Result<Response, StatusCode> {
    let files: [serde_json::Value; 6] = [
        json!({"filename":"预建站文档","filepath":"doc/website.txt"}),
        json!({"filename":"绑定域名","filepath":"doc/bind_domain.txt"}),
        json!({"filename":"广告JS","filepath":"_/static/js/ad.js"}),
        json!({"filename":"关键词库","filepath":"doc/keywords.txt"}),
        json!({"filename":"英文目标","filepath":"doc/target_en.txt"}),
        json!({"filename":"中文目标","filepath":"doc/target_zh.txt"}),
    ];
    // 构造文件路径
    let file_path = params.path.unwrap_or("".to_string());
    let json_result;

    if file_path.len() > 0 {
        // 判断是否存在
        let exists = files.iter().any(|file| {
            if let Some(filepath) = file["filepath"].as_str() {
                filepath == file_path.as_str() // 将 String 转换为 &str 进行比较
            } else {
                false
            }
        });
        if exists {
            // 异步读取文件内容
            match fs::read_to_string(&file_path).await {
                Ok(content) => {
                    json_result =
                        json!({"data":{"content":content},"msg": "文档内容获取 成功", "status": 0});
                }
                Err(e) => {
                    // 处理文件读取错误
                    eprintln!("Failed to read file: {}", e);
                    return Err(StatusCode::NOT_FOUND);
                }
            }
        } else {
            json_result = json!({"data":{"content":""},"msg": "文档内容获取 失败", "status": -1});
        }
    } else {
        json_result = json!({"data":{"items":files},"msg": "文档列表获取 成功", "status": 0});
    }
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

#[derive(Debug, Deserialize)]
pub struct WebsiteQueryParams {
    page: Option<u32>,
    #[serde(rename = "perPage")]
    per_page: Option<u32>,
    is_www: Option<u32>,
    domain: Option<String>,
    root_domain: Option<String>,
    target: Option<String>,
    search_term: Option<String>,
    #[serde(rename = "orderBy")]
    sort_by: Option<String>, // 新增：排序字段
    #[serde(rename = "orderDir")]
    sort_order: Option<String>, // 新增：排序方向（asc/desc）
}

// website_query 方法
pub async fn website_query(
    Query(params): Query<WebsiteQueryParams>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Extension(_my_func): Extension<Arc<MyFunc>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 设置分页参数
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // 构建查询条件
    let mut conditions: HashMap<&str, &str> = HashMap::new();
    match params.is_www {
        Some(1) => {
            conditions.insert("subdomain", "www");
        }
        Some(2) => {
            conditions.insert("subdomain", "!=www");
        }
        _ => {
            // println!("查询所有");
        }
    }
    if let Some(domain) = params.domain.as_ref() {
        if domain.len() > 1 {
            conditions.insert("domain", domain);
        }
    }
    if let Some(root_domain) = params.root_domain.as_ref() {
        if root_domain.len() > 1 {
            conditions.insert("root_domain", root_domain);
        }
    }
    if let Some(target) = params.target.as_ref() {
        if target.len() > 1 {
            conditions.insert("target", target);
        }
    }

    println!("conditions: {:?}", conditions);

    // 设置搜索条件
    let search_term = params.search_term.as_deref();

    // 设置排序参数
    let sort = params.sort_by.as_ref().and_then(|field| {
        if field.is_empty() {
            None // 空字符串返回 None
        } else {
            let direction = params.sort_order.as_ref().map_or("ASC", |order| {
                if order.to_lowercase() == "desc" {
                    "DESC"
                } else {
                    "ASC"
                }
            });
            // 映射 field
            let mapped_field = if field == "website_info.root_domain" {
                "root_domain"
            } else {
                field.as_str()
            };
            Some((mapped_field, direction))
        }
    });

    // 查询所有记录
    let columns = &[];
    let (rows, count) = pgsql
        .fetch_data(
            "website_config",
            columns,
            conditions.clone(),
            None,
            Some(page),
            Some(per_page),
            search_term,
            sort,
        )
        .await?;

    // 转换为 WebsiteConf 结构
    let items: Vec<WebsiteConf> = rows
        .into_iter()
        .map(|row| {
            serde_json::from_value(json!({
                "id": row.get::<i32, _>("id"),
                "website_info": {
                    "domain": row.get::<Option<String>, _>("domain").unwrap_or_default(),
                    "subdomain": row.get::<Option<String>, _>("subdomain").unwrap_or_default(),
                    "root_domain": row.get::<Option<String>, _>("root_domain").unwrap_or_default(),
                    "target": row.get::<Option<String>, _>("target").unwrap_or_default(),
                    "to_lang": row.get::<Option<String>, _>("to_lang").unwrap_or_default(),
                    "title": row.get::<Option<String>, _>("title").unwrap_or_default(),
                    "keywords": row.get::<Option<String>, _>("keywords").unwrap_or_default(),
                    "description": row.get::<Option<String>, _>("description").unwrap_or_default(),
                    "link_mapping": row.get::<bool, _>("link_mapping"),
                },
                "replace_rules": {
                    "replace_mode": row.get::<i32, _>("replace_mode"),
                    "replace_rules_all": row.get::<Option<Vec<String>>, _>("replace_rules_all").unwrap_or_default(),
                    "replace_rules_index": row.get::<Option<Vec<String>>, _>("replace_rules_index").unwrap_or_default(),
                    "replace_rules_page": row.get::<Option<Vec<String>>, _>("replace_rules_page").unwrap_or_default(),
                },
                "mulu_config": {
                    "mulu_tem_max": row.get::<i32, _>("mulu_tem_max"),
                    "mulu_mode": row.get::<Option<String>, _>("mulu_mode").unwrap_or_default(),
                    "mulu_static": row.get::<bool, _>("mulu_static"),
                    "mulu_template": row.get::<Option<Vec<String>>, _>("mulu_template").unwrap_or_default(),
                    "mulu_custom_header": row.get::<Option<Vec<String>>, _>("mulu_custom_header").unwrap_or_default(),
                    "mulu_keywords_file": row.get::<Option<Vec<String>>, _>("mulu_keywords_file").unwrap_or_default(),
                },
                "include_info": {
                    "google_include_info": row.get::<Option<Vec<String>>, _>("google_include_info").unwrap_or_default(),
                    "bing_include_info": row.get::<Option<Vec<String>>, _>("bing_include_info").unwrap_or_default(),
                    "baidu_include_info": row.get::<Option<Vec<String>>, _>("baidu_include_info").unwrap_or_default(),
                    "sogou_include_info": row.get::<Option<Vec<String>>, _>("sogou_include_info").unwrap_or_default(),
                },
                "homepage_update_time": row.get::<i32, _>("homepage_update_time"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at"),
                "updated_at": row.get::<DateTime<Utc>, _>("updated_at"),
            }))
            .expect("Failed to deserialize row")
        })
        .collect();

    // 查询 web_count（domain 不以 www. 开头）
    let mut web_conditions = conditions.clone();
    web_conditions.insert("subdomain", "!=www");
    let (web_rows, web_count) = pgsql
        .fetch_data(
            "website_config",
            &["id"],
            web_conditions,
            None,
            None,
            None,
            search_term,
            None, // 无需排序
        )
        .await?;

    // 查询 www_count（domain 以 www. 开头）
    let mut www_conditions = conditions;
    www_conditions.insert("subdomain", "www");
    let (www_rows, www_count) = pgsql
        .fetch_data(
            "website_config",
            &["id"],
            www_conditions,
            None,
            None,
            None,
            search_term,
            None, // 无需排序
        )
        .await?;

    // 构建 JSON 响应
    let json_result = json!({
        "data": {
            "count": count,
            "web_count": web_rows.len(),
            "www_count": www_rows.len(),
            "items": items,
            "items_count": items.len()
        },
        "msg": "123",
        "status": 0
    });

    Ok(Json(json_result))
}

// #[derive(Deserialize)]
// pub struct WebsiteQueryParams {
//     page: Option<u32>, // 当前页码，默认为 1
//     #[serde(rename = "perPage")]
//     per_page: Option<u32>, // 每页显示的记录数，默认为 20
//     target: Option<String>,
//     search_term: Option<String>,
// }
// pub async fn website_query(
//     Query(params): Query<WebsiteQueryParams>,   // 提取查询参数
//     Extension(client): Extension<Arc<Minio>>,   // MinIO 客户端
//     Extension(my_func): Extension<Arc<MyFunc>>, // 自定义功能模块
// ) -> Result<Json<serde_json::Value>, StatusCode> {
//     // 设置默认分页参数
//     let page = params.page.unwrap_or(1);
//     let per_page = params.per_page.unwrap_or(20);
//     let target = params.target.unwrap_or("".to_string());
//     let search_term = params.search_term.unwrap_or("".to_string());
//     let items_min_count = ((page - 1) * per_page) as usize;
//     let items_max_count = (page * per_page) as usize;

//     // 初始化分页相关变量
//     let mut items = Vec::new(); // 存储当前页的数据
//                                 // let total_count = domains.len(); // 总记录数
//     let mut web_count = 0; // 统计 泛站 域名数量
//     let mut www_count = 0; // 统计 主站 域名数量

//     let mut continuation_token: Option<String> = None;

//     let mut index = 0;

//     loop {
//         // 查询 MinIO
//         let mut args = ListObjectsArgs::default().max_keys(1000).delimiter("/"); // 使用 `/` 作为分隔符
//         if let Some(token) = continuation_token.as_ref() {
//             args = args.continuation_token(token); // 设置分页标记
//         }

//         let result = client.list_objects("config", args).await.map_err(|e| {
//             println!("Error querying MinIO: {}", e);
//             StatusCode::INTERNAL_SERVER_ERROR
//         })?;

//         for prefix in result.common_prefixes {
//             // if items.len()>=items_max_count as usize{
//             //     break;
//             // }
//             let root_domain = prefix.prefix.trim_end_matches('/').to_string();
//             // println!("- {:?}", root_domain);
//             let www_path = format!("{}/www.{}.toml", root_domain, root_domain);

//             let config_files_args = ListObjectsArgs::default()
//                 .prefix(format!("{}/", root_domain))
//                 .max_keys(1000);
//             let config_files_result = client
//                 .list_objects("config", config_files_args)
//                 .await
//                 .map_err(|e| {
//                     println!("Error querying MinIO: {}", e);
//                     StatusCode::INTERNAL_SERVER_ERROR
//                 })?;

//             if config_files_result.contents.len() == 0 {
//                 continue;
//             }

//             index += 1;

//             let mut childrens = Vec::new(); // 存储当前页的数据
//             let mut children_index = 0;

//             let mut item = json!({
//                 "id": Value::Null,
//                 "index":index,
//                 "children": Vec::<Value>::new(),
//                 "domain": Value::Null,
//                 "lang": Value::Null,
//                 "root_domain": Value::Null,
//                 "is_www": false,
//                 "link_mapping": false,
//                 "replace_mode": Value::Null,
//                 "target": Value::Null,
//                 "title": Value::Null,
//                 "keywords": Value::Null,
//                 "description": Value::Null,
//                 "replace_string": Value::Null,
//                 "updated_at": Value::Null
//             });

//             for config_file in config_files_result.contents {
//                 // 进来了表示存在网站配置文件 设置root_domain
//                 if item["root_domain"] == Value::Null {
//                     item["root_domain"] = json!(root_domain);
//                 }
//                 if config_file.key.to_string() == www_path {
//                     www_count += 1;
//                 } else {
//                     web_count += 1;
//                     if items.len() < items_min_count || items.len() >= items_max_count {
//                         // items数量溢出时，不再处理子域名数据
//                         // println!(
//                         //     "{} items数量小于或溢出时，不再处理子域名数据,跳出",
//                         //     config_file.key
//                         // );
//                         continue;
//                     }
//                 }
//                 if let Ok(object) = client.get_object("config", &config_file.key).await {
//                     let content = object.text().await.unwrap();
//                     // 解析 TOML 配置文件
//                     if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//                         // 解析对象键，提取域名和根域名
//                         let domain = match config_file.clone().key.split_once("/") {
//                             Some((_prefix, suffix)) => suffix.trim_end_matches(".toml").to_string(),
//                             None => "".to_string(),
//                         };
//                         if config_file.key.to_string() == www_path {
//                             item["id"] = json!(www_path);
//                             item["domain"] = json!(domain);
//                             item["lang"] = json!(config.info.to_lang);
//                             item["is_www"] = json!(true);
//                             item["link_mapping"] = json!(config.info.link_mapping);
//                             item["replace_mode"] = json!(config.re.replace_mode);
//                             item["target"] = json!(config.info.target);
//                             item["title"] = json!(config.info.title);
//                             item["keywords"] = json!(config.info.keywords);
//                             item["description"] = json!(config.info.description);
//                             item["replace_string"] = json!(my_func.get_replace_string(config.re));
//                             item["updated_at"] = json!(config_file.last_modified);
//                         } else {
//                             children_index += 1;
//                             let children = json!({
//                                 "id": config_file.key,
//                                 "index":format!("┗━ {}.{}",index,children_index),
//                                 "domain": domain,
//                                 "lang": config.info.to_lang,
//                                 "root_domain": root_domain,
//                                 "is_www": false,
//                                 "link_mapping": config.info.link_mapping,
//                                 "replace_mode": config.re.replace_mode,
//                                 "target": config.info.target,
//                                 "title": config.info.title,
//                                 "keywords": config.info.keywords,
//                                 "description": config.info.description,
//                                 "replace_string": my_func.get_replace_string(config.re),
//                                 "updated_at": config_file.last_modified
//                             });
//                             let values: Vec<String> = children
//                                 .as_object()
//                                 .unwrap()
//                                 .values()
//                                 .map(|v| {
//                                     match v {
//                                         serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
//                                         _ => v.to_string(), // 其他类型转换为字符串
//                                     }
//                                 })
//                                 .collect();
//                             // 检查是否有任何一个值包含 search_term
//                             let have_search_term = values.iter().any(|i| i.contains(&search_term));
//                             if have_search_term {
//                                 childrens.push(children);
//                             }
//                         }
//                     }
//                 }
//             }
//             item["children"] = json!(childrens);
//             // root_domain不为空时 才判断写入items
//             if item["root_domain"] != Value::Null || childrens.len() > 0 {
//                 let have_search_term;
//                 if search_term.contains("\n") {
//                     let lines: Vec<&str> = search_term.split("\n").collect();
//                     have_search_term = lines
//                         .iter()
//                         .any(|i| item["domain"].as_str().unwrap().contains(i));
//                 } else {
//                     // 将 JSON 对象的所有值转换为 String，并存储到 Vec<String>
//                     let values: Vec<String> = item
//                         .as_object()
//                         .unwrap()
//                         .values()
//                         .map(|v| {
//                             match v {
//                                 serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
//                                 _ => v.to_string(),                        // 其他类型转换为字符串
//                             }
//                         })
//                         .collect();
//                     // 检查是否有任何一个值包含 search_term
//                     have_search_term = values.iter().any(|i| i.contains(&search_term));
//                 }
//                 if have_search_term {
//                     if target.len() > 0 {
//                         // 处理搜索目标站
//                         println!("target:{}", target);
//                         if let Some(item_target) = item["target"].as_str() {
//                             // println!("Target value without quotes: {}", target); // 输出: example.com (不带双引号)
//                             println!("item[target]:{}", item_target);
//                             if target == item_target.to_string() {
//                                 items.push(item);
//                             }
//                         }
//                     } else {
//                         items.push(item);
//                     }
//                 }
//             }
//         }

//         // 检查是否还有更多对象
//         if !result.is_truncated {
//             // println!("No more objects to list.");
//             break;
//         }
//         // 更新 continuation_token 为下一个分页的起点
//         continuation_token = Some(result.next_continuation_token);
//     }

//     // println!("{:?}",items);
//     let start_num = ((page - 1) * per_page) as usize;
//     let mut end_num = (page * per_page) as usize;
//     if end_num > items.len() {
//         end_num = items.len();
//     }
//     // 构造最终的 JSON 响应
//     let json_result = json!({
//         "data": {
//             "count": items.len(),
//             "web_count": web_count,
//             "www_count": www_count,
//             "items": items[start_num..end_num],
//             "items_count": items[start_num..end_num].len()
//         },
//         "msg": "查询成功",
//         "status": 0
//     });

//     return Ok(Json(json_result));
// }

// 查询 MinIO
// let args = ListObjectsArgs::default().max_keys(1000).delimiter("/"); // 使用 `/` 作为分隔符
// if let Some(token) = continuation_token.as_ref() {
//     args = args.continuation_token(token); // 设置分页标记
// }
// let result = client.list_objects("config", args).await.map_err(|e| {
//     println!("Error querying MinIO: {}", e);
//     StatusCode::INTERNAL_SERVER_ERROR
// })?;

// if result.contents.len() > 999{
//     // 数据超过999 保存下一页标记
//     let token = result.next_continuation_token;

// }

// let mut domains: Vec<String> = Vec::new();
// let mut www_paths: Vec<(usize, String)> = Vec::new();

// for (index, prefix) in result.common_prefixes.into_iter().enumerate() {
//     let domain = prefix.prefix.trim_end_matches('/').to_string();
//     println!("- {:?}", domain);
//     domains.push(domain);
//     let www_path = format!(
//         "{}www.{}.toml",
//         prefix.prefix,
//         prefix.prefix.trim_end_matches('/').to_string()
//     );
//     // www_paths.push(www_path);
//     www_paths.push((index, www_path));
// }

// for (id_num, www_path) in &www_paths[start_num..end_num] {
// for (id_num, www_path) in &www_paths {
//     // 查询 MinIO
//     let prefix_string = format!("{}/", domains[*id_num]);
//     let args = ListObjectsArgs::default()
//         .prefix(prefix_string)
//         .max_keys(1000);
//     let result = client.list_objects("config", args).await.map_err(|e| {
//         println!("Error querying MinIO: {}", e);
//         StatusCode::INTERNAL_SERVER_ERROR
//     })?;
//     let mut childrens = Vec::new(); // 存储当前页的数据
//     let mut www_updated_at = "".to_string();
//     let mut children_index = 0;
//     for object in result.contents {
//         if let Ok(object_data) = client.get_object("config", &object.key).await {
//             let content = object_data.text().await.unwrap();
//             // 解析 TOML 配置文件
//             if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//                 // 解析对象键，提取域名和根域名
//                 let domain = match object.key.split_once("/") {
//                     Some((_prefix, suffix)) => suffix.trim_end_matches(".toml").to_string(),
//                     None => "".to_string(),
//                 };
//                 if &object.key != www_path {
//                     children_index += 1;
//                     web_count += 1;
//                     let children = json!({
//                         "id": object.key,
//                         // "index":format!("{} - {}",id_num+1,children_index),
//                         "index":format!("┗━ {}.{}",id_num+1,children_index),
//                         "domain": domain,
//                         "lang": config.info.to_lang,
//                         "root_domain": domains[*id_num],
//                         "is_www": false,
//                         "link_mapping": config.info.link_mapping,
//                         "replace_mode": config.re.replace_mode,
//                         "target": config.info.target,
//                         "title": config.info.title,
//                         "keywords": config.info.keywords,
//                         "description": config.info.description,
//                         "replace_string": my_func.get_replace_string(config.re),
//                         "updated_at": object.last_modified
//                     });

//                     let values: Vec<String> = children
//                         .as_object()
//                         .unwrap()
//                         .values()
//                         .map(|v| {
//                             match v {
//                                 serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
//                                 _ => v.to_string(), // 其他类型转换为字符串
//                             }
//                         })
//                         .collect();
//                     // 检查是否有任何一个值包含 search_term
//                     let have_search_term = values.iter().any(|i| i.contains(&search_term));
//                     if have_search_term {
//                         childrens.push(children);
//                     }
//                 } else {
//                     www_updated_at = object.last_modified.clone();
//                 }
//             }
//         }
//     }
//     let item;
//     // web_count += childrens.len();
//     if let Ok(object_data) = client.get_object("config", www_path).await {
//         let content = object_data.text().await.unwrap();
//         if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//             item = json!({
//                 "id": www_path,
//                 "index":id_num+1,
//                 "children": childrens,
//                 "domain": format!("www.{}",domains[*id_num]),
//                 "lang": config.info.to_lang,
//                 "root_domain": domains[*id_num],
//                 "is_www": true,
//                 "link_mapping": config.info.link_mapping,
//                 "replace_mode": config.re.replace_mode,
//                 "target": config.info.target,
//                 "title": config.info.title,
//                 "keywords": config.info.keywords,
//                 "description": config.info.description,
//                 "replace_string": my_func.get_replace_string(config.re),
//                 "updated_at": www_updated_at
//             });
//         } else {
//             item = json!({
//                 "id": www_path,
//                 "index":id_num+1,
//                 "children": childrens,
//                 "domain": format!("www.{}",domains[*id_num]),
//                 "lang": "",
//                 "root_domain": domains[*id_num],
//                 "is_www": true,
//                 "link_mapping": false,
//                 "replace_mode": 0,
//                 "target": "",
//                 "title": "",
//                 "keywords": "",
//                 "description": "",
//                 "replace_string": "",
//                 "updated_at": www_updated_at
//             });
//         }
//     } else {
//         item = json!({
//             "id": www_path,
//             "index":id_num+1,
//             "children": childrens,
//             "domain": format!("www.{}",domains[*id_num]),
//             "lang": "",
//             "root_domain": domains[*id_num],
//             "is_www": true,
//             "link_mapping": false,
//             "replace_mode": 0,
//             "target": "",
//             "title": "",
//             "keywords": "",
//             "description": "",
//             "replace_string": "",
//             "updated_at": www_updated_at
//         });
//     }
// 获取对象内容
// if let Ok(object_data) = client.get_object("config", &www_path).await {
//     let content = object_data.text().await.unwrap();
//     // 解析 TOML 配置文件
//     if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//         // 解析对象键，提取域名和根域名
//         let (domain, root_domain, is_www) = (
//             format!("www.{}", www_path.trim_end_matches(".toml")),
//             www_path.trim_end_matches(".toml"),
//             true,
//         );

//         // 构造 JSON 数据
//         let item = json!({
//             "id": www_path,
//             "domain": domain,
//             "lang": config.info.to_lang,
//             "root_domain": root_domain,
//             "is_www": is_www,
//             "link_mapping": config.info.link_mapping,
//             "replace_mode": config.re.replace_mode,
//             "target": config.info.target,
//             "title": config.info.title,
//             "keywords": config.info.keywords,
//             "description": config.info.description,
//             "replace_string": my_func.get_replace_string(config.re),
//             "updated_at": object.last_modified
//         });

//         items.push(item); // 添加到结果列表
// }

// match params.parent_id {
//     Some(parent_id) => {
//         // 泛站查询
//         println!("parent_id exists: {}", parent_id);
//         // 获取对象内容
//         if let Ok(object_data) = client.get_object("config", &parent_id).await {
//             let content = object_data.text().await.unwrap();
//             // 解析 TOML 配置文件
//             if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//                 // 解析对象键，提取域名和根域名
//                 let (domain, root_domain, is_www) = (
//                     format!("www.{}", parent_id.trim_end_matches(".toml")),
//                     parent_id.trim_end_matches(".toml"),
//                     true,
//                 );
//                 // 查询 MinIO
//                 let prefix_string = format!("{}/", parent_id.trim_end_matches(".toml"));
//                 let args = ListObjectsArgs::default()
//                     .prefix(prefix_string)
//                     .max_keys(1000);
//                 let result = client.list_objects("config", args).await.map_err(|e| {
//                     println!("Error querying MinIO: {}", e);
//                     StatusCode::INTERNAL_SERVER_ERROR
//                 })?;
//                 let mut childrens = Vec::new(); // 存储当前页的数据
//                                                 // 遍历返回的对象
//                 for object in result.contents {
//                     // 获取对象内容
//                     if let Ok(object_data) = client.get_object("config", &object.key).await {
//                         let content = object_data.text().await.unwrap();
//                         // 解析 TOML 配置文件
//                         if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//                             let children = json!({
//                                 "id": object.key,
//                                 "domain": domain,
//                                 "lang": config.info.to_lang,
//                                 "root_domain": root_domain,
//                                 "is_www": false,
//                                 "link_mapping": config.info.link_mapping,
//                                 "replace_mode": config.re.replace_mode,
//                                 "target": config.info.target,
//                                 "title": config.info.title,
//                                 "keywords": config.info.keywords,
//                                 "description": config.info.description,
//                                 "replace_string": my_func.get_replace_string(config.re),
//                                 "updated_at": object.last_modified
//                             });
//                             childrens.push(children);
//                         }
//                     }
//                 }

//                 // 构造 JSON 数据
//                 let data = json!({
//                     "id": parent_id,
//                     "children": childrens,
//                     "domain": domain,
//                     "lang": config.info.to_lang,
//                     "root_domain": root_domain,
//                     "is_www": is_www,
//                     "link_mapping": config.info.link_mapping,
//                     "replace_mode": config.re.replace_mode,
//                     "target": config.info.target,
//                     "title": config.info.title,
//                     "keywords": config.info.keywords,
//                     "description": config.info.description,
//                     "replace_string": my_func.get_replace_string(config.re),
//                     // "updated_at": object_data.last_modified
//                 });
//                 // 构造最终的 JSON 响应
//                 let json_result = json!({
//                     "data": data,
//                     "msg": "查询成功",
//                     "status": 0
//                 });
//                 return Ok(Json(json_result));
//             }
//         }
//     }
//     None => {
//         println!("parent_id does not exist");
//         // 主站查询
//         // 设置默认分页参数
//         let page = params.page.unwrap_or(1);
//         let per_page = params.per_page.unwrap_or(20);
//         let params_is_www = params.is_www.unwrap_or(1);

//         // 初始化分页相关变量
//         let mut items = Vec::new(); // 存储当前页的数据
//         let mut continuation_token = None; // 分页标记
//         let mut total_count = 0; // 总记录数
//         let mut www_count = 0; // 统计 www 域名数量

//         // 计算分页的起始和结束位置
//         let start_index = (page - 1) * per_page;
//         let end_index = start_index + per_page;

//         loop {
//             // 构造分页查询参数
//             let args = if let Some(ref token) = continuation_token {
//                 // 如果 continuation_token 不为 None，则使用它
//                 ListObjectsArgs::default()
//                     .max_keys(per_page.try_into().unwrap())
//                     .continuation_token(token)
//             } else {
//                 // 如果 continuation_token 为 None，则不传递 continuation_token 参数
//                 ListObjectsArgs::default().max_keys(per_page.try_into().unwrap())
//             };

//             // 查询 MinIO
//             let result = client.list_objects("config", args).await.map_err(|e| {
//                 println!("Error querying MinIO: {}", e);
//                 StatusCode::INTERNAL_SERVER_ERROR
//             })?;

//             // 遍历返回的对象
//             for object in result.contents {
//                 // 如果当前记录在分页范围内，则处理
//                 if total_count >= start_index && items.len() < per_page as usize {
//                     // 获取对象内容
//                     if let Ok(object_data) = client.get_object("config", &object.key).await {
//                         let content = object_data.text().await.unwrap();

//                         // 解析 TOML 配置文件
//                         if let Ok(config) = toml::from_str::<WebsiteConf>(&content) {
//                             // 解析对象键，提取域名和根域名
//                             let (domain, root_domain, is_www) = match object.key.split_once("/")
//                             {
//                                 Some((_prefix, suffix)) => {
//                                     if params_is_www == 1 {
//                                         continue; // 跳过非 www 域名
//                                     }
//                                     (
//                                         suffix.trim_end_matches(".toml").to_string(),
//                                         _prefix,
//                                         false,
//                                     )
//                                 }
//                                 None => {
//                                     www_count += 1; // 统计 www 域名
//                                     if params_is_www == 2 {
//                                         continue; // 跳过 www 域名
//                                     }
//                                     (
//                                         format!("www.{}", object.key.trim_end_matches(".toml")),
//                                         object.key.trim_end_matches(".toml"),
//                                         true,
//                                     )
//                                 }
//                             };

//                             // 构造 JSON 数据
//                             let item = json!({
//                                 "id": object.key,
//                                 "defer": true,
//                                 "domain": domain,
//                                 "lang": config.info.to_lang,
//                                 "root_domain": root_domain,
//                                 "is_www": is_www,
//                                 "link_mapping": config.info.link_mapping,
//                                 "replace_mode": config.re.replace_mode,
//                                 "target": config.info.target,
//                                 "title": config.info.title,
//                                 "keywords": config.info.keywords,
//                                 "description": config.info.description,
//                                 "replace_string": my_func.get_replace_string(config.re),
//                                 "updated_at": object.last_modified
//                             });

//                             items.push(item); // 添加到结果列表
//                         }
//                     }
//                 }

//                 // 更新总记录数
//                 total_count += 1;

//                 // 如果已达到分页范围上限，停止处理
//                 if items.len() >= per_page as usize {
//                     break;
//                 }
//             }

//             // 检查是否还有更多数据
//             if result.is_truncated && items.len() < per_page as usize {
//                 continuation_token = Some(result.next_continuation_token);
//             } else {
//                 break; // 没有更多数据，退出循环
//             }
//         }

//         // 构造最终的 JSON 响应
//         let json_result = json!({
//             "data": {
//                 "count": total_count,
//                 "web_count": total_count - www_count,
//                 "www_count": www_count,
//                 "items": items,
//                 "items_count": items.len()
//             },
//             "msg": "查询成功",
//             "status": 0
//         });

//         return Ok(Json(json_result));
//     }
// }

// 构造最终的 JSON 响应
//     let json_result = json!({
//         "msg": "查询失败",
//         "status": -1
//     });

//     Ok(Json(json_result))
// }
// pub async fn website_query(
//     Query(params): Query<WebsiteQueryParams>, // 提取查询参数
//     Extension(client): Extension<Arc<Minio>>, // MinIO 客户端
//     Extension(my_func): Extension<Arc<MyFunc>>, // 自定义功能模块
// ) -> Result<Json<serde_json::Value>, StatusCode> {
//     // 设置默认分页参数
//     let page = params.page.unwrap_or(1);
//     let per_page = params.per_page.unwrap_or(20);
//     let params_is_www = params.is_www.unwrap_or(1);

//     // 初始化分页相关变量
//     let mut items = Vec::new(); // 存储当前页的数据
//     let mut continuation_token = None; // 分页标记
//     let mut total_count = 0; // 总记录数
//     let mut www_count = 0; // 统计 www 域名数量

//     // 计算分页的起始位置
//     let start_index = (page - 1) * per_page;
//     let mut fetched_count = 0; // 已获取的记录数

//     loop {
//         // 构造分页查询参数
//         // 动态构建 ListObjectsArgs
// let args = if let Some(ref token) = continuation_token {
//     // 如果 continuation_token 不为 None，则使用它
//     ListObjectsArgs::default()
//         .max_keys(per_page.try_into().unwrap())
//         .continuation_token(token)
// } else {
//     // 如果 continuation_token 为 None，则不传递 continuation_token 参数
//     ListObjectsArgs::default()
//         .max_keys(per_page.try_into().unwrap())
// };

//         // 查询 MinIO
//         match client.list_objects("config",args).await {
//             Ok(result) => {
//                 total_count += result.contents.len(); // 累计总记录数

//                 // 遍历返回的对象
//                 for object in result.contents {
//                     if total_count > (start_index + per_page).try_into().unwrap() {
//                         // 如果已达到分页范围上限，停止处理
//                         break;
//                     }
//                     if total_count >= start_index.try_into().unwrap() {
//                         // 如果当前记录在目标分页范围内
//                         match client.get_object("config", &object.key).await {
//                             Ok(object_data) => {
//                                 let content = object_data.text().await.unwrap(); // 获取对象内容
//                                 let parsed_config: Result<WebsiteConf, toml::de::Error> =
//                                     toml::from_str(&content);

//                                 match parsed_config {
//                                     Ok(config) => {
//                                         // 解析对象键，提取域名和根域名
//                                         let (domain, root_domain, is_www) = match object.key.split_once("/") {
//                                             Some((_prefix, suffix)) => {
//                                                 if params_is_www == 1 {
//                                                     continue; // 跳过非 www 域名
//                                                 }
//                                                 (
//                                                     suffix.trim_end_matches(".toml").to_string(), // 域名
//                                                     _prefix, // 根域名
//                                                     false, // 是否为 www 域名
//                                                 )
//                                             }
//                                             None => {
//                                                 www_count += 1; // 统计 www 域名
//                                                 if params_is_www == 2 {
//                                                     continue; // 跳过 www 域名
//                                                 }
//                                                 (
//                                                     format!("www.{}", object.key.trim_end_matches(".toml")), // 域名
//                                                     object.key.trim_end_matches(".toml"), // 根域名
//                                                     true, // 是否为 www 域名
//                                                 )
//                                             }
//                                         };

//                                         // 构造 JSON 数据
//                                         let item = json!({
//                                             "id": object.key,
//                                             "domain": domain,
//                                             "lang": config.info.to_lang,
//                                             "root_domain": root_domain,
//                                             "is_www": is_www,
//                                             "link_mapping": config.info.link_mapping,
//                                             "replace_mode": config.re.replace_mode,
//                                             "target": config.info.target,
//                                             "title": config.info.title,
//                                             "keywords": config.info.keywords,
//                                             "description": config.info.description,
//                                             "replace_string": my_func.get_replace_string(config.re),
//                                             "updated_at": object.last_modified
//                                         });

//                                         items.push(item); // 添加到结果列表
//                                     }
//                                     Err(e) => {
//                                         println!("Error parsing TOML: {}", e);
//                                     }
//                                 }
//                             }
//                             Err(_) => {
//                                 println!("{} 没有配置文件", object.key);
//                             }
//                         }
//                     }

//                     // fetched_count += 1; // 累计已处理的记录数
//                     // if fetched_count >= start_index + per_page {
//                     //     // 如果已达到分页范围上限，停止处理
//                     //     break;
//                     // }
//                 }

//                 // 检查是否还有更多数据
//                 if result.is_truncated {
//                     continuation_token = Some(result.next_continuation_token);
//                 } else {
//                     break; // 没有更多数据，退出循环
//                 }
//             }
//             Err(e) => {
//                 println!("Error querying MinIO: {}", e);
//                 return Err(StatusCode::INTERNAL_SERVER_ERROR);
//             }
//         }
//     }

//     // 构造最终的 JSON 响应
//     let json_result = json!({
//         "data": {
//             "count": total_count,
//             "web_count": total_count - www_count,
//             "www_count": www_count,
//             "items": items,
//             "items_count": items.len()
//         },
//         "msg": "查询成功",
//         "status": 0
//     });

//     Ok(Json(json_result))
// }

// pub async fn website_query(
//     Query(params): Query<WebsiteQueryParams>,
//     Extension(my_func): Extension<Arc<MyFunc>>,
//     Extension(client): Extension<Arc<Minio>>,
//     req: Request,
// ) -> Result<Response, StatusCode> {
//     // info!("minio path: {url}");
//     // 获取完整的 URI
//     // let path = url.trim_matches('/');
//     // let (bucket_name, object_name) = uri.split_once("/");
//     // let (bucket_name, object_name) = uri.trim_matches('/').split_once('/').unwrap_or((&uri, ""));
//     // 去掉 URI 开头和结尾的 '/'
//     // let uri = uri.trim_matches('/');
//     let params_is_www = params.is_www.unwrap_or(1);
//     // // 按第一个 '/' 分割字符串
//     // let (bucket_name, object_name) = path
//     //     .split_once('/')
//     //     .map(|(b, o)| (b, o)) // 如果分割成功，返回 (bucket, object)
//     //     .unwrap_or_else(|| (path, "")); // 如果分割失败，返回 (uri, "")

//     // 如果对象名称为空，返回错误
//     // if object_name.is_empty() {
//     // 表示获取列表
//     // 获取对象的元数据
//     // """"""

//     match client
//         .list_objects("config", ListObjectsArgs::default().max_keys(100))
//         .await
//     {
//         Ok(result) => {
//             println!("{:?}", result);
//             let objects = result.contents; // 请根据实际情况调整字段名
//                                            // 提取对象键并构造 JSON 数据
//             println!("{:?}", objects);
//             let count = result.key_count;
//             let mut items = Vec::new(); // 使用 Vec::new() 初始化 items
//             let mut www_count = 0;

//             for i in objects {
//                 println!("{}", i.key); // 打印 key
//                 match client.get_object("config", &i.key).await {
//                     Ok(object) => {
//                         let content = object.text().await.unwrap();
//                         println!("content: {}", content);
//                         // 解析 TOML
//                         let parsed_config: Result<WebsiteConf, toml::de::Error> =
//                             toml::from_str(&content);
//                         match parsed_config {
//                             Ok(config) => {
//                                 println!("rules: {:?}", config.re);
//                                 let domain;
//                                 let root_domain;
//                                 let is_www;
//                                 if let Some((_prefix, suffix)) = i.key.split_once("/") {
//                                     if params_is_www == 1 {
//                                         continue;
//                                     }
//                                     // 返回 suffix，并去掉可能的 ".toml" 后缀
//                                     domain = suffix.trim_end_matches(".toml").to_string();
//                                     root_domain = _prefix;
//                                     is_www = false;
//                                 } else {
//                                     www_count += 1;
//                                     if params_is_www == 2 {
//                                         continue;
//                                     }
//                                     root_domain = i.key.trim_end_matches(".toml");
//                                     domain = format!("www.{}", root_domain);
//                                     is_www = true;
//                                 }
//                                 let item = json!({
//                                     "id": i.key,
//                                     "domain": domain,1
//                                     "lang": config.info.to_lang,
//                                     "root_domain": root_domain,
//                                     "is_www": is_www,
//                                     "link_mapping":config.info.link_mapping,
//                                     "replace_mode":config.re.replace_mode,
//                                     "target": config.info.target,
//                                     "title": config.info.title,
//                                     "keywords": config.info.keywords,
//                                     "description": config.info.description,
//                                     "replace_string": my_func.get_replace_string(config.re),
//                                     "updated_at": i.last_modified
//                                 });
//                                 items.push(item); // 将 item 添加到 items 中
//                             }
//                             Err(e) => {
//                                 println!("Error parsing TOML: {}", e);
//                             }
//                         }
//                     }
//                     Err(_) => {
//                         println!("{} 没有配置文件", i.key);
//                     }
//                 }
//             }
//             let json_result = json!({
//                 "data": {
//                     "count": count,
//                     "web_count": count,
//                     "www_count": www_count,
//                     "items": items,
//                     "items_count": items.len()
//                 },
//                 "msg": "123",
//                 "status": 0
//             });
//             // 返回 JSON 响应
//             return Ok(Response::builder()
//                 .header("Content-Type", "application/json")
//                 .body(Body::from(json_result.to_string()))
//                 .unwrap());
//         }
//         Err(_) => return Err(StatusCode::NOT_FOUND),
//     }
// }

// #[derive(Deserialize)]
// pub struct WebsiteInsertParams {
//     file_path: Option<i32>,
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct WebsiteInsertData {
    pub domain: String,
    pub lang: String,
    pub target: String,
    pub title: String,
    pub keywords: String,
    pub description: String,
    pub target_replace: String,
    pub replace_rules_all: Option<Vec<String>>,
    pub replace_rules_index: Option<Vec<String>>,
    pub replace_rules_page: Option<Vec<String>>,
    pub link_mapping: bool,
    pub replace_mode: i32,
    pub mulu_static: bool,
    pub homepage_update_time: i32,
    pub mulu_tem_max: i32,
    pub mulu_mode: Option<String>,
    pub mulu_custom_header: Option<Vec<String>>,
    pub mulu_keywords_file: Option<Vec<String>>,
}
// #[axum::debug_handler]
pub async fn website_insert(
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Json(data): Json<WebsiteInsertData>,
) -> Result<Response, StatusCode> {
    // 处理target_replace
    println!("{}", data.target_replace);
    println!("{}", REPALCE_CONTENT.to_string());

    if data.target_replace.len() > 2 && data.target_replace != REPALCE_CONTENT.to_string() {
        let (_lang, replace_file) = match data.target.split_once('|') {
            Some((lang, replace_file)) => {
                println!("Language: {}", lang);
                println!("Replace File: {}", replace_file);
                (lang.to_string(), format!("{}.toml", replace_file))
            }
            None => {
                println!("Invalid input format");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let target_replace_rules = my_func
            .load_replace_string(data.target_replace)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let target_replace_config_template = format!(
            r#"all = {:?}
    index = {:?}
    page = {:?}"#,
            target_replace_rules.全局替换,
            target_replace_rules.首页替换,
            target_replace_rules.内页替换,
        );
        // 将字符串转换为字节数据
        let file_content = target_replace_config_template.clone().into_bytes();
        // 上传文件到 MinIO
        match client
            .put_object("replace", &replace_file, file_content.into())
            .await
        {
            Ok(_) => {
                println!("{} replace文件编辑成功", replace_file);
            }
            Err(e) => {
                println!("{} replace文件编辑失败: {}", replace_file, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // 处理config
    let domain_info = domain_info_from_domain(&data.domain);
    // 创建 HashMap，包含表中所有字段
    let mut datas = HashMap::new();

    // 必填字段
    datas.insert("domain", domain_info["full_domain"].as_str()); // 唯一域名
    datas.insert("root_domain", domain_info["root_domain"].as_str()); // 根域名

    // 可选字段
    datas.insert("subdomain", domain_info["subdomain"].as_str()); // 子域名
    datas.insert("target", data.target.as_str()); // 目标站
    datas.insert("to_lang", data.lang.as_str()); // 语言（英文）
    datas.insert("title", data.title.as_str()); // 页面标题
    datas.insert("keywords", data.keywords.as_str()); // 关键词
    datas.insert("description", data.description.as_str()); // 描述

    // 布尔值和整数字段
    datas.insert(
        "link_mapping",
        if data.link_mapping { "true" } else { "false" },
    ); // 启用链接映射

    let replace_mode = data.replace_mode.to_string();
    datas.insert("replace_mode", replace_mode.as_str()); // 替换模式设为 1

    datas.insert(
        "mulu_static",
        if data.mulu_static { "true" } else { "false" },
    ); // 不启用目录静态化

    let mulu_tem_max: String = data.mulu_tem_max.to_string();
    datas.insert("mulu_tem_max", mulu_tem_max.as_str());

    let homepage_update_time: String = data.homepage_update_time.to_string();
    datas.insert("homepage_update_time", homepage_update_time.as_str()); // 首页每3600秒更新

    // 插入 HashMap（使用静态字符串作为键）
    match data.replace_rules_all {
        Some(ref replace_rules_all) => {
            datas.insert(
                "replace_rules_all",
                Box::leak(MyFunc::vec_to_pg_array(replace_rules_all).into_boxed_str())
                    as &'static str,
            );
        }
        None => {
            datas.insert("replace_rules_all", "{}"); // 目录自定义头
        }
    }
    match data.replace_rules_index {
        Some(ref replace_rules_index) => {
            datas.insert(
                "replace_rules_index",
                Box::leak(MyFunc::vec_to_pg_array(replace_rules_index).into_boxed_str())
                    as &'static str,
            );
        }
        None => {
            datas.insert("replace_rules_index", "{}"); // 目录自定义头
        }
    }
    match data.replace_rules_page {
        Some(ref replace_rules_page) => {
            datas.insert(
                "replace_rules_page",
                Box::leak(MyFunc::vec_to_pg_array(replace_rules_page).into_boxed_str())
                    as &'static str,
            );
        }
        None => {
            datas.insert("replace_rules_page", "{}"); // 目录自定义头
        }
    }

    match data.mulu_custom_header {
        Some(ref mulu_custom_header) => {
            datas.insert(
                "mulu_custom_header",
                Box::leak(MyFunc::vec_to_pg_array(mulu_custom_header).into_boxed_str())
                    as &'static str,
            );
        }
        None => {
            datas.insert("mulu_custom_header", "{}"); // 目录自定义头
        }
    }

    match data.mulu_keywords_file {
        Some(ref mulu_keywords_file) => {
            datas.insert(
                "mulu_keywords_file",
                Box::leak(MyFunc::vec_to_pg_array(mulu_keywords_file).into_boxed_str())
                    as &'static str,
            );
        }
        None => {
            datas.insert("mulu_custom_header", "{}"); // 目录自定义头
        }
    }

    datas.insert("mulu_template", "{}"); // 目录模板
    datas.insert("google_include_info", "{}"); // 谷歌收录页面
    datas.insert("bing_include_info", "{}"); // 必应收录页面
    datas.insert("baidu_include_info", "{}"); // 百度收录（空数组）
    datas.insert("sogou_include_info", "{}"); // 搜狗收录（空数组）

    // 目录模式（多选值）
    datas.insert(
        "mulu_mode",
        data.mulu_mode.as_ref().map_or("", |s| s.as_str()),
    );

    match pgsql
        .insert_or_create_config("website_config", datas, false)
        .await
    {
        Ok(()) => {
            println!("{} 网站配置 插入成功", &data.domain);
            let r_mes = format!("【{}】网站配置 新建成功", data.domain);
            // let new_pgsql = pgsql.clone();
            // 自动清空缓存
            cache_delete(
                Query(CacheDeleteParams {
                    domains: data.domain,
                    ids: "".to_string(),
                }),
                Extension(pgsql),
                // Extension(minio_client),
            )
            .await?;
            let json_result = json!({"msg": r_mes,"status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
        Err(status) => {
            let r_mes;
            if status == StatusCode::CONFLICT {
                println!("存在配置文件 {} 跳过新建", domain_info["full_domain"]);
                r_mes = format!(
                    "【{}】网站配置 新建失败，已存在配置文件，请直接编辑",
                    data.domain
                );
            } else {
                println!("{} 配置文件 插入失败", &data.domain);
                r_mes = format!("【{}】网站配置 新建失败", data.domain);
            }
            let json_result = json!({"msg": r_mes,"status": -1});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebsiteCreateData {
    domain: String,
    lang: String,
    target: String,
    title: String,
    keywords: String,
    description: String,
    target_replace: String,
    replace_string: String,
    link_mapping: bool,
    replace_mode: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebsiteCreateContentData {
    lang: String,
    content: String,
    over_write: bool,
    target_replace_over_write: bool,
    link_mapping: bool,
    replace_mode: i32,
    mulu_static: bool,
    homepage_update_time: i32,
    mulu_tem_max: i32,
    mulu_mode: Option<String>,
    mulu_custom_header: Option<Vec<String>>,
    mulu_keywords_file: Option<Vec<String>>,
}

// #[axum::debug_handler]
pub async fn website_create(
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Json(json_data): Json<WebsiteCreateContentData>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines: Vec<&str> = json_data.content.split("\n").collect();

    let mut create_count = 0;
    let mut jump_count = 0;

    // 打印每一行
    for (index, line) in lines.iter().enumerate() {
        // println!("Line {}: {}", index + 1, line);
        let parts: Vec<&str> = line.split("___").collect();
        // println!("{:?}", parts);
        if parts.len() != 7 {
            let json_result =
                json!({"msg": format!("第{}行 数据错误 请检查", index + 1), "status": -1});
            return Ok(Json(json_result));
        }
        let data = WebsiteCreateData {
            domain: parts[0].to_string(),
            lang: json_data.lang.clone(),
            target: parts[1].to_string(),
            link_mapping: json_data.link_mapping,
            title: parts[2].to_string(),
            keywords: parts[3].to_string(),
            description: parts[4].to_string(),
            replace_mode: json_data.replace_mode.clone(),
            target_replace: parts[5].to_string(),
            replace_string: parts[6].to_string(),
        };

        // 处理target_replace
        println!("{}", data.target_replace);
        // println!("{}", REPALCE_CONTENT.to_string());
        // 跳过已存在

        if data.target_replace.len() > 2 && data.target_replace != REPALCE_CONTENT.to_string() {
            let (_lang, replace_file) = match data.target.split_once('|') {
                Some((lang, replace_file)) => {
                    println!("Language: {}", lang);
                    println!("Replace File: {}", replace_file);
                    (lang.to_string(), format!("{}.toml", replace_file))
                }
                None => {
                    println!("Invalid input format");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
            let mut jump_create_target_replace = false;
            if !json_data.target_replace_over_write {
                // 判断是否存在
                match client.stat_object("replace", &replace_file).await {
                    Ok(Some(_)) => {
                        println!("存在文件 {} 跳过新建target_replace", replace_file);
                        jump_create_target_replace = true;
                    }
                    Ok(None) | Err(_) => {
                        println!("不存在文件 {} 开始新建target_replace", replace_file);
                    }
                }
            }
            if !jump_create_target_replace {
                let target_replace_rules = my_func
                    .load_replace_string(data.target_replace)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let target_replace_config_template = format!(
                    r#"all = {:?}
index = {:?}
page = {:?}"#,
                    target_replace_rules.全局替换,
                    target_replace_rules.首页替换,
                    target_replace_rules.内页替换,
                );
                // 将字符串转换为字节数据
                let file_content = target_replace_config_template.clone().into_bytes();
                // 上传文件到 MinIO
                match client
                    .put_object("replace", &replace_file, file_content.into())
                    .await
                {
                    Ok(_) => {
                        println!("{} replace文件编辑成功", replace_file);
                    }
                    Err(e) => {
                        println!("{} replace文件编辑失败: {}", replace_file, e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
        }

        // 处理config
        let replace_rules = my_func
            .load_replace_string(data.replace_string)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 将字符串转换为字节数据
        // let file_content = config_template.clone().into_bytes();
        let domain_info = domain_info_from_domain(&data.domain);
        // 创建 HashMap，包含表中所有字段
        let mut datas = HashMap::new();

        // 必填字段
        datas.insert("domain", domain_info["full_domain"].as_str()); // 唯一域名
        datas.insert("root_domain", domain_info["root_domain"].as_str()); // 根域名

        // 可选字段
        datas.insert("subdomain", domain_info["subdomain"].as_str()); // 子域名
        datas.insert("target", data.target.as_str()); // 目标站
        datas.insert("to_lang", data.lang.as_str()); // 语言（英文）
        datas.insert("title", data.title.as_str()); // 页面标题
        datas.insert("keywords", data.keywords.as_str()); // 关键词
        datas.insert("description", data.description.as_str()); // 描述

        // 布尔值和整数字段
        datas.insert(
            "link_mapping",
            if data.link_mapping { "true" } else { "false" },
        ); // 启用链接映射

        let replace_mode = data.replace_mode.to_string();
        datas.insert("replace_mode", replace_mode.as_str()); // 替换模式设为 1

        datas.insert(
            "mulu_static",
            if json_data.mulu_static {
                "true"
            } else {
                "false"
            },
        ); // 不启用目录静态化

        let mulu_tem_max: String = json_data.mulu_tem_max.to_string();
        datas.insert("mulu_tem_max", mulu_tem_max.as_str());

        let homepage_update_time: String = json_data.homepage_update_time.to_string();
        datas.insert("homepage_update_time", homepage_update_time.as_str()); // 首页每3600秒更新

        // 数组字段（使用 PostgreSQL 数组字面量格式） 转换 Vec<String> 为 PostgreSQL 数组字面量
        let all_rules = MyFunc::vec_to_pg_array(&replace_rules.全局替换);
        let index_rules = MyFunc::vec_to_pg_array(&replace_rules.首页替换);
        let page_rules = MyFunc::vec_to_pg_array(&replace_rules.内页替换);

        // 插入 HashMap（使用静态字符串作为键）
        datas.insert(
            "replace_rules_all",
            Box::leak(all_rules.into_boxed_str()) as &'static str,
        );
        datas.insert(
            "replace_rules_index",
            Box::leak(index_rules.into_boxed_str()) as &'static str,
        );
        datas.insert(
            "replace_rules_page",
            Box::leak(page_rules.into_boxed_str()) as &'static str,
        );

        match json_data.mulu_custom_header {
            Some(ref mulu_custom_header) => {
                datas.insert(
                    "mulu_custom_header",
                    Box::leak(MyFunc::vec_to_pg_array(mulu_custom_header).into_boxed_str())
                        as &'static str,
                );
            }
            None => {
                datas.insert("mulu_custom_header", "{}"); // 目录自定义头
            }
        }

        match json_data.mulu_keywords_file {
            Some(ref mulu_keywords_file) => {
                datas.insert(
                    "mulu_keywords_file",
                    Box::leak(MyFunc::vec_to_pg_array(mulu_keywords_file).into_boxed_str())
                        as &'static str,
                );
            }
            None => {
                datas.insert("mulu_custom_header", "{}"); // 目录自定义头
            }
        }

        datas.insert("mulu_template", "{}"); // 目录模板
                                             // datas.insert("mulu_custom_header", "{}"); // 目录自定义头
                                             // datas.insert("mulu_keywords_file", "{}"); // 关键词库文件
        datas.insert("google_include_info", "{}"); // 谷歌收录页面
        datas.insert("bing_include_info", "{}"); // 必应收录页面
        datas.insert("baidu_include_info", "{}"); // 百度收录（空数组）
        datas.insert("sogou_include_info", "{}"); // 搜狗收录（空数组）

        // 目录模式（多选值）
        datas.insert(
            "mulu_mode",
            json_data.mulu_mode.as_ref().map_or("", |s| s.as_str()),
        );

        match pgsql
            .insert_or_create_config("website_config", datas, json_data.over_write)
            .await
        {
            Ok(()) => {
                println!("{} 配置文件 插入成功", &data.domain);
                create_count += 1;
                let new_pgsql = pgsql.clone();
                // 自动清空缓存
                cache_delete(
                    Query(CacheDeleteParams {
                        domains: data.domain,
                        ids: "".to_string(),
                    }),
                    Extension(new_pgsql),
                )
                .await?;
            }
            Err(status) => {
                if status == StatusCode::CONFLICT {
                    println!("存在配置文件 {} 跳过新建", domain_info["full_domain"]);
                    jump_count += 1;
                } else {
                    println!("{} 配置文件 插入失败", &data.domain);
                }
            }
        }
    }
    let json_result;
    if jump_count > 0 {
        json_result = json!({"msg": format!("跳过已存在网站: {}个 本次建站: {}个 ",jump_count,create_count), "status": 0});
    } else {
        json_result = json!({"msg": format!("本次建站: {}个 ",create_count), "status": 0});
    }

    return Ok(Json(json_result));
}

#[derive(Deserialize)]
pub struct WebsitePutParams {
    id: String,
}

#[derive(Deserialize, Serialize, Debug)]
// pub struct WebsitePutData {
//     domain: String,
//     lang: String,
//     target: String,
//     title: String,
//     keywords: String,
//     description: String,
//     replace_string: String,
//     target_replace: String,
//     link_mapping: bool,
//     replace_mode: i32,
// }
pub struct WebsitePutData {
    pub website_info: WebsiteInfo,
    pub replace_rules: ReplaceRules,
    pub mulu_config: MuluConfig,
    pub homepage_update_time: i32,
    pub target_replace: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn website_update(
    Query(params): Query<WebsitePutParams>,
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    // Extension(minio_client): Extension<Arc<MinioClient>>,
    Json(data): Json<WebsitePutData>,
) -> Result<Response, StatusCode> {
    // let id = params.id;

    // // 获取文件的元数据
    // let metadata = client
    //     .stat_object("config", &file)
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Failed to get file metadata: {}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?
    //     .ok_or_else(|| {
    //         eprintln!("File not found: {}", file);
    //         StatusCode::NOT_FOUND // 返回 404 状态码表示文件不存在
    //     })?;

    // // 获取文件的最后修改时间（假设是字符串）
    // let modified_time_str = metadata.last_modified(); // 假设返回的是字符串
    // println!("Last modified time: {}", modified_time_str);
    // let modified_time = DateTime::parse_from_rfc2822(modified_time_str)
    //     .map_err(|e| {
    //         eprintln!("Failed to parse last modified time: {}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?
    //     .with_timezone(&Utc); // 转换为 UTC 时间

    // // 获取当前时间
    // let current_time = Utc::now();

    // // 计算时间差
    // let time_diff = current_time.signed_duration_since(modified_time);
    // let time_diff_secs = time_diff.num_seconds();

    // // 如果时间差小于 60 秒，则返回失败
    // if time_diff_secs < 60 {
    //     let json_result = json!({
    //         "msg": "网站配置文件 编辑失败：距离上次修改时间不足60秒",
    //         "status": -1
    //     });

    //     return Ok(Response::builder()
    //         .header("Content-Type", "application/json")
    //         .body(Body::from(json_result.to_string()))
    //         .unwrap());
    // }

    // 处理 target_replace
    if data.target_replace.len() > 2 && data.target_replace != REPALCE_CONTENT.to_string() {
        let (_lang, replace_file) = match data.website_info.target.split_once('|') {
            Some((lang, replace_file)) => {
                println!("Language: {}", lang);
                println!("Replace File: {}", replace_file);
                (lang.to_string(), format!("{}.toml", replace_file))
            }
            None => {
                println!("Invalid input format");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let target_replace_rules = my_func
            .load_replace_string(data.target_replace)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let target_replace_config_template = format!(
            r#"all = {:?}
    index = {:?}
    page = {:?}"#,
            target_replace_rules.全局替换,
            target_replace_rules.首页替换,
            target_replace_rules.内页替换,
        );
        // 将字符串转换为字节数据
        let file_content = target_replace_config_template.clone().into_bytes();
        // 上传文件到 MinIO
        match client
            .put_object("replace", &replace_file, file_content.into())
            .await
        {
            Ok(_) => {
                println!("{} replace文件编辑成功", replace_file);
            }
            Err(e) => {
                println!("{} replace文件编辑失败: {}", replace_file, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // 处理 config
    // 创建 HashMap，包含表中所有字段
    let mut datas = HashMap::new();

    // 必填字段
    datas.insert("domain", data.website_info.domain.as_str()); // 唯一域名
    datas.insert("root_domain", data.website_info.root_domain.as_str()); // 根域名

    // 可选字段
    // datas.insert("subdomain", domain_info["subdomain"].as_str()); // 子域名
    datas.insert("target", data.website_info.target.as_str()); // 目标站
    datas.insert("to_lang", data.website_info.to_lang.as_str()); // 语言（英文）
    datas.insert("title", data.website_info.title.as_str()); // 页面标题
    datas.insert("keywords", data.website_info.keywords.as_str()); // 关键词
    datas.insert("description", data.website_info.description.as_str()); // 描述

    // 布尔值和整数字段
    datas.insert(
        "link_mapping",
        if data.website_info.link_mapping {
            "true"
        } else {
            "false"
        },
    ); // 启用链接映射

    let replace_mode = data.replace_rules.replace_mode.to_string();
    datas.insert("replace_mode", replace_mode.as_str()); // 替换模式设为 1

    datas.insert(
        "mulu_static",
        if data.mulu_config.mulu_static {
            "true"
        } else {
            "false"
        },
    ); // 不启用目录静态化

    let mulu_tem_max: String = data.mulu_config.mulu_tem_max.to_string();
    datas.insert("mulu_tem_max", mulu_tem_max.as_str());

    let homepage_update_time: String = data.homepage_update_time.to_string();
    datas.insert("homepage_update_time", homepage_update_time.as_str()); // 首页每3600秒更新

    // 数组字段（使用 PostgreSQL 数组字面量格式） 转换 Vec<String> 为 PostgreSQL 数组字面量
    let all_rules = MyFunc::vec_to_pg_array(&data.replace_rules.all);
    let index_rules = MyFunc::vec_to_pg_array(&data.replace_rules.index);
    let page_rules = MyFunc::vec_to_pg_array(&data.replace_rules.page);

    let mulu_template = MyFunc::vec_to_pg_array(&data.mulu_config.mulu_template);
    let mulu_custom_header = MyFunc::vec_to_pg_array(&data.mulu_config.mulu_custom_header);
    let mulu_keywords_file = MyFunc::vec_to_pg_array(&data.mulu_config.mulu_keywords_file);

    // 插入 HashMap（使用静态字符串作为键）
    datas.insert(
        "replace_rules_all",
        Box::leak(all_rules.into_boxed_str()) as &'static str,
    );
    datas.insert(
        "replace_rules_index",
        Box::leak(index_rules.into_boxed_str()) as &'static str,
    );
    datas.insert(
        "replace_rules_page",
        Box::leak(page_rules.into_boxed_str()) as &'static str,
    );
    datas.insert(
        "mulu_template",
        Box::leak(mulu_template.into_boxed_str()) as &'static str,
    );
    datas.insert(
        "mulu_custom_header",
        Box::leak(mulu_custom_header.into_boxed_str()) as &'static str,
    );
    datas.insert(
        "mulu_keywords_file",
        Box::leak(mulu_keywords_file.into_boxed_str()) as &'static str,
    );

    // datas.insert("mulu_template", "{}"); // 目录模板
    // datas.insert("mulu_custom_header", "{}"); // 目录自定义头
    // datas.insert("mulu_keywords_file", "{}"); // 关键词库文件

    // 目录模式（多选值）
    datas.insert("mulu_mode", data.mulu_config.mulu_mode.as_str()); // 目录模式为 404 和自定义头

    match pgsql
        .insert_or_create_config("website_config", datas, true)
        .await
    {
        Ok(()) => {
            println!("{} 网站配置 插入成功", &data.website_info.domain);
            let r_mes = format!(
                "ID:{}【{}】网站配置 编辑成功",
                params.id, data.website_info.domain
            );
            // 自动清空缓存
            cache_delete(
                Query(CacheDeleteParams {
                    domains: data.website_info.domain,
                    ids: "".to_string(),
                }),
                Extension(pgsql),
                // Extension(minio_client),
            )
            .await?;
            let json_result = json!({"msg": r_mes,"status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
        Err(e) => {
            println!(
                "{}、{} 网站配置 编辑失败: {}",
                params.id, data.website_info.domain, e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    //     let replace_rules = my_func
    //         .load_replace_string(data.replace_string)
    //         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    //     let link_mapping_string = if data.link_mapping { "true" } else { "false" };
    //     let config_template = format!(
    //         r#"[website_info]
    // target = "{}"
    // to_lang = "{}"
    // title = "{}"
    // description = "{}"
    // keywords = "{}"
    // link_mapping = {}

    // [replace_rules]
    // replace_mode = {}
    // all = {:?}
    // index = {:?}
    // page = {:?}"#,
    //         data.target,
    //         data.lang,
    //         data.title,
    //         data.description,
    //         data.keywords,
    //         link_mapping_string,
    //         data.replace_mode,
    //         replace_rules.全局替换,
    //         replace_rules.首页替换,
    //         replace_rules.内页替换,
    //     );

    //     // 将字符串转换为字节数据
    //     let file_content = config_template.clone().into_bytes();
    //     // 上传文件到 MinIO
    //     match client
    //         .put_object("config", &file, file_content.into())
    //         .await
    //     {
    //         Ok(_) => {
    //             println!("{} 配置文件编辑成功", file);

    //             // 自动清空缓存
    //             cache_delete(
    //                 Query(CacheDeleteParams {
    //                     domains: data.domain,
    //                     ids: "".to_string(),
    //                 }),
    //                 Extension(pgsql),
    //                 // Extension(minio_client),
    //             )
    //             .await?;

    //             let json_result = json!({"msg": "网站配置文件 编辑成功", "status": 0});

    //             return Ok(Response::builder()
    //                 .header("Content-Type", "application/json")
    //                 .body(Body::from(json_result.to_string()))
    //                 .unwrap());
    //         }
    //         Err(e) => {
    //             println!("{} 网站配置文件 编辑失败: {}", file, e);
    //             return Err(StatusCode::INTERNAL_SERVER_ERROR);
    //         }
    //     }
}

// pub async fn website_update(
//     Query(params): Query<WebsitePutParams>,
//     Extension(my_func): Extension<Arc<MyFunc>>,
//     Extension(client): Extension<Arc<Minio>>,
//     Json(data): Json<WebsitePutData>,
// ) -> Result<Response, StatusCode> {
//     // 处理target_replace
//     if data.target_replace != REPALCE_CONTENT.to_string() {
//         let (lang, replace_file) = match data.target.split_once('|') {
//             Some((lang, replace_file)) => {
//                 println!("Language: {}", lang);
//                 println!("Replace File: {}", replace_file);
//                 (lang.to_string(), format!("{}.toml", replace_file))
//             }
//             None => {
//                 println!("Invalid input format");
//                 return Err(StatusCode::INTERNAL_SERVER_ERROR);
//             }
//         };
//         let target_replace_rules = my_func
//             .load_replace_string(data.target_replace)
//             .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//         let target_replace_config_template = format!(
//             r#"all = {:?}
//     index = {:?}
//     page = {:?}"#,
//             target_replace_rules.全局替换,
//             target_replace_rules.首页替换,
//             target_replace_rules.内页替换,
//         );
//         // 将字符串转换为字节数据
//         let file_content = target_replace_config_template.clone().into_bytes();
//         // 上传文件到 MinIO
//         match client
//             .put_object("replace", &replace_file, file_content.into())
//             .await
//         {
//             Ok(_) => {
//                 println!("{} replace文件编辑成功", replace_file);
//             }
//             Err(e) => {
//                 println!("{} replace文件编辑失败: {}", replace_file, e);
//                 return Err(StatusCode::INTERNAL_SERVER_ERROR);
//             }
//         }
//     }

//     // 处理config
//     let file = params.file;
//     let replace_rules = my_func
//         .load_replace_string(data.replace_string)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     let link_mapping_string = if data.link_mapping { "true" } else { "false" };
//     let config_template = format!(
//         r#"[website_info]
// target = "{}"
// to_lang = "{}"
// title = "{}"
// description = "{}"
// keywords = "{}"
// link_mapping = {}

// [replace_rules]
// replace_mode = {}
// all = {:?}
// index = {:?}
// page = {:?}"#,
//         data.target,
//         data.lang,
//         data.title,
//         data.description,
//         data.keywords,
//         link_mapping_string,
//         data.replace_mode,
//         replace_rules.全局替换,
//         replace_rules.首页替换,
//         replace_rules.内页替换,
//     );

//     // 将字符串转换为字节数据
//     let file_content = config_template.clone().into_bytes();
//     // 上传文件到 MinIO
//     match client
//         .put_object("config", &file, file_content.into())
//         .await
//     {
//         Ok(_) => {
//             println!("{} 配置文件编辑成功", file);
//             cache_delete(
//                 Query(CacheDeleteParams {
//                     domain: data.domain,
//                 }),
//                 Extension(client),
//             )
//             .await?;

//             let json_result = json!({"msg": "编辑网站 成功", "status": 0});

//             return Ok(Response::builder()
//                 .header("Content-Type", "application/json")
//                 .body(Body::from(json_result.to_string()))
//                 .unwrap());
//         }
//         Err(e) => {
//             println!("{} 配置文件编辑失败: {}", file, e);
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     }
// }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct WebsiteData {
//     files: String,
// }
#[derive(Deserialize)]
pub struct WebsiteDeleteParams {
    files: String, // 假设 files 是一个包含文件路径
}

// #[axum::debug_handler]
pub async fn website_delete(
    Query(params): Query<WebsiteDeleteParams>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
) -> Result<Response, StatusCode> {
    let mut deleted_count = 0;
    let id_list: Vec<&str> = params
        .files
        .split(",")
        .map(|id| id.trim())
        .filter(|id| !id.is_empty())
        .collect();
    if id_list.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    for id in id_list {
        // 验证 ID 是数字
        if !id.chars().all(|c| c.is_digit(10)) {
            return Err(StatusCode::BAD_REQUEST);
        }

        match pgsql
            .delete_data("website_config", HashMap::from([("id", id)]))
            .await
        {
            Ok(_) => {
                deleted_count += 1;
            }
            Err(e) => {
                println!("删除 {} 的 ID {} 失败: {}", "website_config", id, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let json_result = json!({"msg": format!("删除网站{}个 成功",deleted_count), "status": 0});
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

#[derive(Deserialize)]
pub struct CacheDomainsParams {
    is_www: String,
    page: Option<u32>, // 当前页码，默认为 1
    #[serde(rename = "perPage")]
    per_page: Option<u32>, // 每页显示的记录数，默认为 20
    // target_lib: Option<String>,
    domain: Option<String>,
}

pub async fn cache_domains(
    Query(params): Query<CacheDomainsParams>,
    // Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    // Json(data): Json<WebsiteData>,
) -> Result<Response, StatusCode> {
    let starts_with = if params.is_www == "true" {
        Some("www")
        // Some("www")
    } else {
        None
    };
    let ends_with = params.domain.as_deref();
    match pgsql
        .get_paginated_tables(
            starts_with,
            ends_with,
            params.page.unwrap_or(1).into(),
            params.per_page.unwrap_or(20).into(),
        )
        .await
    {
        Ok((table_names, total)) => {
            let items: Vec<_> = table_names
                .iter()
                .filter_map(|(index, table_name)| {
                    let is_www = if table_name.starts_with("www__") {
                        true
                    } else {
                        false
                    };
                    // if params.is_www == "true" && !is_www {
                    //     None
                    // } else {
                    let domain = table_name.replace("__", ".").replace("_", ".");
                    Some(json!({
                        "index": index,
                        "domain": domain,
                        "is_www": is_www,
                    }))
                    // }
                })
                .collect();

            let json_result = json!({
                "data": {
                    "count": total,
                    "items": items,
                    "items_count": table_names.len(),
                },
                "msg": "查询成功",
                "status": 0
            });
            // let json_result = json!({"msg": "获取缓存列表 成功", "status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
        Err(e) => {
            println!("获取缓存列表 失败: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[derive(Deserialize)]
pub struct CacheDeleteParams {
    domains: String,
    #[serde(default)] // 如果未提供 ids，则为空字符串
    ids: String, // 新增 ids 字段
}

pub async fn cache_delete(
    Query(params): Query<CacheDeleteParams>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
) -> Result<Response, StatusCode> {
    let domains = params.domains.trim(); // 移除首尾空格
    let json_result;

    // 验证 domains 非空
    if domains.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if domains.contains(",") {
        let mut deleted_count = 0;
        let mut errors = Vec::new();
        let domains_list: Vec<&str> = domains
            .split(",")
            .map(|d| d.trim())
            .filter(|d| !d.is_empty())
            .collect();

        // 验证域名列表非空
        if domains_list.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 批量处理域名
        for domain in domains_list {
            // 验证域名格式（简单检查）
            if domain
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_')
            {
                errors.push(format!("无效域名: {}", domain));
                continue;
            }

            // 生成 table_name
            let domain_info = domain_info_from_domain(&domain);
            let table_name = format!(
                "{}__{}",
                domain_info["subdomain"], domain_info["root_domain"]
            )
            .replace(".", "_");

            // 删除表格
            match pgsql.drop_table(&table_name, false).await {
                Ok(_) => {
                    println!("表格 {} 删除成功", table_name);
                    deleted_count += 1;
                }
                Err(e) => {
                    println!("表格 {} 删除失败: {}", table_name, e);
                    errors.push(format!("删除表格 {} 失败: {}", table_name, e));
                }
            }
        }
        // 根据结果生成响应
        json_result = json!({
            "msg": format!("批量清空缓存 {}个", deleted_count),
            "status": 0
        });
    } else {
        // 处理单个域名
        let mut deleted_count = 0;
        let ids = params.ids.trim();

        // 验证域名格式
        if domains
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 生成 table_name
        let domain_info = domain_info_from_domain(&domains);
        let table_name = format!(
            "{}__{}",
            domain_info["subdomain"], domain_info["root_domain"]
        )
        .replace(".", "_");

        if !ids.is_empty() {
            let id_list: Vec<&str> = ids
                .split(",")
                .map(|id| id.trim())
                .filter(|id| !id.is_empty())
                .collect();
            if id_list.is_empty() {
                return Err(StatusCode::BAD_REQUEST);
            }

            for id in id_list {
                // 验证 ID 是数字
                if !id.chars().all(|c| c.is_digit(10)) {
                    return Err(StatusCode::BAD_REQUEST);
                }

                match pgsql
                    .delete_data(&table_name, HashMap::from([("id", id)]))
                    .await
                {
                    Ok(_) => {
                        deleted_count += 1;
                    }
                    Err(e) => {
                        println!("删除 {} 的 ID {} 失败: {}", table_name, id, e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
            json_result = json!({
                "msg": format!("【{}】删除缓存 {}条",domains, deleted_count),
                "status": 0
            });
        } else {
            match pgsql.drop_table(&table_name, false).await {
                Ok(_) => {
                    println!("表格 {} 删除成功", table_name);
                    // deleted_count += 1;
                }
                Err(e) => {
                    println!("表格 {} 删除失败: {}", table_name, e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
            json_result = json!({
                "msg": format!("【{}】清空缓存 成功", domains),
                "status": 0
            });
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

#[derive(Deserialize)]
pub struct CacheQueryParams {
    page: Option<u32>, // 当前页码，默认为 1
    #[serde(rename = "perPage")]
    per_page: Option<u32>, // 每页显示的记录数，默认为 20
    // file: Option<String>,
    // is_mapping: Option<String>,
    domain: Option<String>,
    search_term: Option<String>,
    page_type: Option<String>,
    uri: Option<String>,
}

pub async fn cache_query(
    Query(params): Query<CacheQueryParams>, // 提取查询参数
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    // Extension(my_func): Extension<Arc<MyFunc>>, // 自定义功能模块
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 设置默认分页参数
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // let page_type = params.page_type.unwrap_or("".to_string());
    // let uri = params.uri.unwrap_or("".to_string());

    let domain = if let Some(domain) = params.domain {
        domain
    } else {
        let json_result = json!({
            "data": {
                "count": 0,
                "items": [],
                "items_count": 0,
            },
            "msg": "查询成功",
            "status": 0
        });
        return Ok(Json(json_result));
    };
    let search_term = params.search_term;
    let domain_info = domain_info_from_domain(&domain);
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");

    let mut conditions: HashMap<&str, &str> = HashMap::new();
    if let Some(ref page_type) = params.page_type {
        if !page_type.is_empty() {
            conditions.insert("page_type", page_type);
        }
    }

    if let Some(ref uri) = params.uri {
        if !uri.is_empty() {
            conditions.insert("uri", uri);
        }
    }

    match pgsql
        .fetch_data(
            &table_name,
            &[
                "id",
                "url",
                "page_type",
                "uri",
                "target",
                "updated_at",
                "title",
                "keywords",
                "description",
                "domain",
            ],
            // HashMap::from([("cache_path", cache_path.as_str())]),
            conditions,
            None,
            Some(page),
            Some(per_page),
            search_term.as_deref(),
            None,
        )
        .await
    {
        Ok((rows, total)) => {
            // 将 PgRow 转换为可序列化的格式
            let items: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "id": row.get::<i32, _>("id"),
                        "url": row.get::<String, _>("url"),
                        "page_type": row.get::<String, _>("page_type"),
                        "uri": row.get::<String, _>("uri"),
                        "target": row.get::<String, _>("target"),
                        "title": row.get::<String, _>("title"),
                        "keywords": row.get::<String, _>("keywords"),
                        "description": row.get::<String, _>("description"),
                        "domain": row.get::<String, _>("domain"),
                        "updated_at": row.get::<DateTime<Utc>, _>("updated_at"),
                    })
                })
                .collect();

            // println!("{:?}", items);

            let json_result = json!({
                "data": {
                    "count": total,
                    "items": items,
                    "items_count": items.len(),
                },
                "msg": "查询成功",
                "status": 0
            });
            Ok(Json(json_result))
        } // 表存在，直接返回数据
        Err(status) => {
            println!("表不存在: {}", status);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[derive(Deserialize)]
pub struct ReplaceQueryParams {
    domain: String,
}

pub async fn replace_query(
    Query(params): Query<ReplaceQueryParams>,
    // Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    // req: Request,
) -> Result<Response, StatusCode> {
    let (_lang, replace_file) = match params.domain.split_once('|') {
        Some((lang, replace_file)) => {
            // println!("Language: {}", lang);
            // println!("Replace File: {}", replace_file);
            (lang.to_string(), format!("{}.toml", replace_file))
        }
        None => {
            println!("Invalid input format");
            // return Err(StatusCode::INTERNAL_SERVER_ERROR);
            // let repalce_content = "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'";
            let json_result = json!({"data":{"target_replace":REPALCE_CONTENT},"msg": "获取默认replace配置 成功", "status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
    };
    match client.get_object("replace", &replace_file).await {
        Ok(object) => {
            let content = object.text().await.unwrap();
            // println!("content: {}", content);
            // 解析 TOML
            let parsed_replace_conf: Result<TargetReplaceRules, toml::de::Error> =
                toml::from_str(&content);
            match parsed_replace_conf {
                Ok(replace_conf) => {
                    // println!("replace_conf: {:?}", replace_conf);
                    // println!("{} 配置文件编辑成功", file);
                    // let 全局替换 = replace_conf.all
                    // let repalce_content = "全局替换:\n  - '待替换字符串1 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'";
                    let mut replace_content = String::from("全局替换:\n");
                    for i in replace_conf.all {
                        replace_content.push_str(&format!("  - '{}'\n", i));
                    }
                    replace_content.push_str("首页替换:\n");
                    for i in replace_conf.index {
                        replace_content.push_str(&format!("  - '{}'\n", i));
                    }
                    replace_content.push_str("内页替换:\n");
                    for i in replace_conf.page {
                        replace_content.push_str(&format!("  - '{}'\n", i));
                    }

                    let json_result = json!({"data":{"target_replace":replace_content.trim()},"msg": "获取replace配置 成功", "status": 0});

                    return Ok(Response::builder()
                        .header("Content-Type", "application/json")
                        .body(Body::from(json_result.to_string()))
                        .unwrap());
                }
                Err(e) => {
                    println!("Error parsing TOML: {} {}", replace_file, e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        Err(_) => {
            println!("{} 没有配置文件，返回默认replace配置", replace_file);
            // let repalce_content = "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'";
            let json_result = json!({"data":{"target_replace":REPALCE_CONTENT},"msg": "获取默认replace配置 成功", "status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
    }
}

#[derive(Deserialize)]
pub struct TargetQueryParams {
    page: Option<u32>, // 当前页码，默认为 1
    #[serde(rename = "perPage")]
    per_page: Option<u32>, // 每页显示的记录数，默认为 20
    file: Option<String>,
    domain: Option<String>,
    target_lib: Option<String>,
    search_term: Option<String>,
}
pub async fn target_query(
    Query(params): Query<TargetQueryParams>, // 提取查询参数
    Extension(client): Extension<Arc<Minio>>, // MinIO 客户端
                                             // Extension(my_func): Extension<Arc<MyFunc>>, // 自定义功能模块
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 设置默认分页参数
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let file = params.file.unwrap_or("".to_string());
    let domain = params.domain.unwrap_or("".to_string());
    let target_lib = params.target_lib.unwrap_or("target-zh".to_string());
    let search_term = params.search_term.unwrap_or("".to_string());
    let items_min_count = ((page - 1) * per_page) as usize;
    let items_max_count = (page * per_page) as usize;

    // 初始化分页相关变量
    let mut items = Vec::new(); // 存储当前页的数据
    let mut total_count = 0; // 总记录数
    let mut index = 0;

    let mut continuation_token: Option<String> = None;

    loop {
        // 查询 MinIO
        let prefix_string = format!("{}/", domain);
        let mut args = ListObjectsArgs::default()
            .prefix(prefix_string)
            .max_keys(1000);
        if let Some(token) = continuation_token.as_ref() {
            args = args.continuation_token(token); // 设置分页标记
        }
        let result = client.list_objects(&target_lib, args).await.map_err(|e| {
            println!("Error querying MinIO: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        total_count += result.contents.len();

        // 获取childrens数据
        for object in result.contents {
            index += 1;
            let mut children = json!({
                "id": object.key,
                "index":index,
                "url": Value::Null,
                "status_code": Value::Null,
                "target_lib": Value::Null,
                "updated_at": Value::Null
            });
            if items.len() < items_min_count || items.len() >= items_max_count {
                // items数量溢出时，不再处理子域名数据
                // println!(
                //     "{} items数量小于或溢出时，不再处理详细数据,跳出",
                //     object.key
                // );
                items.push(children);
                continue;
            }
            match client.stat_object(&target_lib, &object.key).await {
                Ok(Some(object_stat)) => {
                    if let Some((ct, url)) = object_stat.content_type().split_once("|") {
                        let status_code = if ct.to_string().chars().all(|c| c.is_ascii_digit()) {
                            ct
                        } else {
                            "200"
                        };
                        children["url"] = json!(url);
                        children["status_code"] = json!(status_code);
                        children["target_lib"] = json!(target_lib);
                        children["updated_at"] = json!(object_stat.last_modified());
                        let values: Vec<String> = children
                            .as_object()
                            .unwrap()
                            .values()
                            .map(|v| {
                                match v {
                                    serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
                                    _ => v.to_string(), // 其他类型转换为字符串
                                }
                            })
                            .collect();
                        // 检查是否有任何一个值包含 search_term
                        let have_search_term = values.iter().any(|i| i.contains(&search_term));
                        if have_search_term {
                            items.push(children);
                            // if file.len() == 0 && search_term.len() == 0 {
                            //     break;
                            // }
                        }
                    };
                }
                Ok(None) | Err(_) => {
                    println!("fuck");
                }
            }
        }
        // 检查是否还有更多对象
        if !result.is_truncated {
            // println!("No more objects to list.");
            break;
        }
        // 更新 continuation_token 为下一个分页的起点
        continuation_token = Some(result.next_continuation_token);
    }

    let start_num = ((page - 1) * per_page) as usize;
    let mut end_num = (page * per_page) as usize;
    if end_num > items.len() {
        end_num = items.len();
    }

    // 构造最终的 JSON 响应
    let json_result = json!({
        "data": {
            "count": items.len(),
            "site_count": items.len(),
            "total_count": total_count,
            "items": items[start_num..end_num],
            "items_count": items[start_num..end_num].len(),
            "target_lib": target_lib,
            "domain": domain,
        },
        "msg": "查询成功",
        "status": 0
    });
    return Ok(Json(json_result));
}

#[derive(Deserialize)]
pub struct TargetDomainsParams {
    page: Option<u32>, // 当前页码，默认为 1
    #[serde(rename = "perPage")]
    per_page: Option<u32>, // 每页显示的记录数，默认为 20
    target_lib: Option<String>,
    domain: Option<String>,
}

pub async fn target_domains(
    Query(params): Query<TargetDomainsParams>,
    // Json(data): Json<WebsiteData>
    // Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    // Json(data): Json<WebsiteData>,
) -> Result<Response, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let params_target_lib = params.target_lib.unwrap_or("zh".to_string());
    let search_domain = params.domain.unwrap_or("".to_string());
    let target_lib = if params_target_lib.starts_with("target-") {
        params_target_lib
    } else {
        format!("target-{}", params_target_lib)
    };
    let mut index = 0;
    let mut items = Vec::new(); // 存储当前页的数据
    let mut continuation_token: Option<String> = None;
    let (target_lib_full_name, target_lib_name, target_lib_level) = match target_lib.as_str() {
        // 使用 as_str() 转换为 &str
        "target-zh" => ("中文 [zh]", "中文", "danger"),
        "target-en" => ("英文 [en]", "英文", "warning"),
        "target-en2zh" => ("英译中 [en2zh]", "英中", "success"),
        "target-zh2en" => ("中译英 [zh2en]", "中英", "info"),
        _ => ("未知", "未知", "default"), // 默认情况
    };

    loop {
        // 查询 MinIO
        let mut args = ListObjectsArgs::default().max_keys(1000).delimiter("/"); // 使用 `/` 作为分隔符
        if let Some(token) = continuation_token.as_ref() {
            args = args.continuation_token(token); // 设置分页标记
        }
        let result = client.list_objects(&target_lib, args).await.map_err(|e| {
            println!("Error querying MinIO: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        for prefix in result.common_prefixes {
            index += 1;
            if search_domain.len() > 1 && !prefix.prefix.contains(&search_domain) {
                continue;
            }
            let domain = prefix.prefix.trim_end_matches('/').to_string();

            // 判断是否为空文件夹
            let check_null_result = client
                .list_objects(
                    &target_lib,
                    ListObjectsArgs::default().prefix(prefix.prefix).max_keys(1),
                )
                .await
                .map_err(|e| {
                    println!("Error querying MinIO: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            if check_null_result.contents.len() == 0 {
                index -= 1;
                continue;
            }

            let item = json!({
                "target_lib": target_lib,
                "domain": domain,
                "index":index,
                "target_lib_name":target_lib_name,
                "target_lib_level":target_lib_level,
            });
            items.push(item);
        }
        // 检查是否还有更多对象
        if !result.is_truncated {
            // println!("No more objects to list.");
            break;
        }
        // 更新 continuation_token 为下一个分页的起点
        continuation_token = Some(result.next_continuation_token);
    }

    let start_num = ((page - 1) * per_page) as usize;
    let mut end_num = (page * per_page) as usize;
    if end_num > items.len() {
        end_num = items.len();
    }

    // 构造最终的 JSON 响应
    let json_result = json!({
        "data": {
            "count": items.len(),
            "items": items[start_num..end_num],
            "items_count": items[start_num..end_num].len(),
            "target_lib":target_lib,
            "target_lib_full_name":target_lib_full_name,
        },
        "msg": "查询成功",
        "status": 0
    });
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

#[derive(Deserialize)]
pub struct TargetDeleteParams {
    bucket: String,        // 假设 files 是一个包含文件路径
    files: Option<String>, // 假设 files 是一个包含文件路径
    domain: Option<String>,
}

// #[axum::debug_handler]
pub async fn target_delete(
    Query(params): Query<TargetDeleteParams>,
    // Json(data): Json<WebsiteData>
    // Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    // Extension(minio_client): Extension<Arc<MinioClient>>,
    // Json(data): Json<WebsiteData>,
) -> Result<Response, StatusCode> {
    let domain = params.domain.unwrap_or("".to_string());
    let files = params.files.unwrap_or("".to_string());
    let mut file_paths = Vec::new();
    if domain.len() > 1 {
        let mut continuation_token: Option<String> = None;
        loop {
            let mut target_files_args = ListObjectsArgs::default()
                .prefix(format!("{}/", domain))
                .max_keys(1000);
            if let Some(token) = continuation_token.as_ref() {
                target_files_args = target_files_args.continuation_token(token);
                // 设置分页标记
            }
            let target_files_result = client
                .list_objects(&params.bucket, target_files_args)
                .await
                .map_err(|e| {
                    println!("Error querying MinIO: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            file_paths.extend(
                target_files_result
                    .contents
                    .iter()
                    .map(|obj| obj.key.clone()),
            );
            // 检查是否还有更多对象
            if !target_files_result.is_truncated {
                // println!("No more objects to list.");
                break;
            }
            // 更新 continuation_token 为下一个分页的起点
            continuation_token = Some(target_files_result.next_continuation_token);
        }
    } else {
        // 将 files 字段按逗号拆分为多个文件路径
        file_paths.extend(files.split(',').map(|s| s.trim().to_string()));
    }
    let mut count = 0;
    for file_path in file_paths {
        match client.remove_object(&params.bucket, &file_path).await {
            Ok(_) => {
                println!("文件 {} 删除成功", file_path);
                count += 1;
            }
            Err(e) => {
                println!("文件 {} 删除失败: {}", file_path, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let json_result = json!({"msg": format!("删除目标页面 {}条",count), "status": 0});
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap())
}

// #[derive(Deserialize)]
// pub struct CacheQueryParams_ {
//     page: Option<u32>, // 当前页码，默认为 1
//     #[serde(rename = "perPage")]
//     per_page: Option<u32>, // 每页显示的记录数，默认为 20
//     file: Option<String>,
//     is_mapping: Option<String>,
//     search_term: Option<String>,
// }
// pub async fn cache_query_(
//     Query(params): Query<CacheQueryParams_>, // 提取查询参数
//     Extension(client): Extension<Arc<Minio>>, // MinIO 客户端
//                                              // Extension(my_func): Extension<Arc<MyFunc>>, // 自定义功能模块
// ) -> Result<Json<serde_json::Value>, StatusCode> {
//     // 设置默认分页参数
//     let page = params.page.unwrap_or(1);
//     let per_page = params.per_page.unwrap_or(20);
//     let is_mapping = params.is_mapping.unwrap_or("".to_string());
//     let file = params.file.unwrap_or("".to_string());
//     let search_term = params.search_term.unwrap_or("".to_string());

//     // 查询 MinIO
//     let args = ListObjectsArgs::default().max_keys(1000).delimiter("/"); // 使用 `/` 作为分隔符
//     let result = client.list_objects("cache", args).await.map_err(|e| {
//         println!("Error querying MinIO: {}", e);
//         StatusCode::INTERNAL_SERVER_ERROR
//     })?;
//     let mut domains: Vec<String> = Vec::new();
//     let mut index_paths: Vec<(usize, String)> = Vec::new();

//     for (index, prefix) in result.common_prefixes.into_iter().enumerate() {
//         let domain = prefix.prefix.trim_end_matches('/').to_string();
//         // println!("- {:?}", domain);
//         domains.push(domain);
//         let index_path = format!("{}index.html", prefix.prefix);
//         if file.len() > 0 {
//             if file == index_path {
//                 index_paths.push((index, index_path));
//                 break;
//             }
//         } else {
//             index_paths.push((index, index_path));
//         }
//     }
//     // println!("{:?}", domains);
//     // println!("{:?}", index_paths);

//     // 初始化分页相关变量
//     let mut items = Vec::new(); // 存储当前页的数据
//     let mut total_count = 0; // 总记录数

//     for (id_num, index_path) in &index_paths {
//         // 查询 MinIO
//         let prefix_string = format!("{}/", domains[*id_num]);
//         let args = ListObjectsArgs::default()
//             .prefix(prefix_string)
//             .max_keys(1000);
//         let result = client.list_objects("cache", args).await.map_err(|e| {
//             println!("Error querying MinIO: {}", e);
//             StatusCode::INTERNAL_SERVER_ERROR
//         })?;
//         // println!("{:?}", result);
//         let mut childrens = Vec::new(); // 存储当前页的数据
//         let mut children_index = 0;

//         total_count += result.contents.len();
//         for object in result.contents {
//             if &object.key == index_path {
//                 continue;
//             }
//             match client.stat_object("cache", &object.key).await {
//                 Ok(Some(object_stat)) => {
//                     let content_type = object_stat.content_type();
//                     let url_str = content_type.trim_start_matches("uri:");
//                     let parts: Vec<&str> = url_str.splitn(2, "|").collect();

//                     if url_str.contains("quanjibocai") {
//                         println!("quanjibocai   :{} {:?}", url_str, parts);
//                     }
//                     let (map_link, url_link) = if parts.len() == 2 {
//                         (parts[0], parts[1])
//                     } else {
//                         ("", url_str)
//                     };
//                     // let (map_link, url_link) = if let Some((link, url)) = url_str.split_once("|") {
//                     //     (link, url)
//                     // } else {
//                     //     ("", url_str)
//                     // };

//                     if is_mapping == "true".to_string() {
//                         if map_link.len() == 0 {
//                             continue;
//                         }
//                     } else if is_mapping == "false".to_string() {
//                         if map_link.len() > 0 {
//                             continue;
//                         }
//                     }
//                     children_index += 1;
//                     let children = json!({
//                         "id": object.key,
//                         "index":format!("┗━ {}.{}",id_num+1,children_index),
//                         "url": url_link,
//                         "mapping_url":map_link,
//                         "is_mapping": map_link.len()>0,
//                         "domain": domains[*id_num],
//                         "updated_at": object_stat.last_modified()
//                     });
//                     let values: Vec<String> = children
//                         .as_object()
//                         .unwrap()
//                         .values()
//                         .map(|v| {
//                             match v {
//                                 serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
//                                 _ => v.to_string(),                        // 其他类型转换为字符串
//                             }
//                         })
//                         .collect();

//                     // 检查是否有任何一个值包含 search_term
//                     let have_search_term = values.iter().any(|i| i.contains(&search_term));
//                     // println!("content_type {} {:?}", content_type, values);
//                     // println!("have_search_term: {}", have_search_term);
//                     if have_search_term {
//                         childrens.push(children);
//                         if file.len() == 0 && search_term.len() == 0 {
//                             break;
//                         }
//                     }
//                 }
//                 Ok(None) | Err(_) => {
//                     println!("fuck");
//                 }
//             }
//         }

//         let mut updated_at = "".to_string();
//         let mut map_link = "".to_string();
//         let mut url_link = "".to_string();
//         match client.stat_object("cache", index_path).await {
//             Ok(Some(object_stat)) => {
//                 // total_count += 1;
//                 let content_type = object_stat.content_type();
//                 let url_str = content_type.trim_start_matches("uri:");
//                 let (map_link_, url_link_) = if let Some((link, url)) = url_str.split_once("|") {
//                     (link, url)
//                 } else {
//                     ("", url_str)
//                 };
//                 updated_at = object_stat.last_modified().to_string();
//                 map_link = map_link_.to_string();
//                 url_link = url_link_.to_string();
//             }
//             Ok(None) | Err(_) => {
//                 println!("{} {} 不存在", "cache", index_path);
//             }
//         };

//         let item;
//         if file.len() > 0 || search_term.len() > 0 {
//             item = json!({
//                 "id": index_path,
//                 "index":id_num+1,
//                 "children": childrens,
//                 "url": url_link,
//                 "mapping_url":map_link,
//                 "is_mapping": map_link.len()>0,
//                 "domain": domains[*id_num],
//                 "updated_at": updated_at
//             });
//         } else {
//             item = json!({
//                 "id": index_path,
//                 "index":id_num+1,
//                 "defer": childrens.len()>0,
//                 "url": url_link,
//                 "mapping_url":map_link,
//                 "is_mapping": map_link.len()>0,
//                 "domain": domains[*id_num],
//                 "updated_at": updated_at
//             });
//         }

//         // 将 JSON 对象的所有值转换为 String，并存储到 Vec<String>
//         let values: Vec<String> = item
//             .as_object()
//             .unwrap()
//             .values()
//             .map(|v| {
//                 match v {
//                     serde_json::Value::String(s) => s.clone(), // 直接使用字符串值
//                     _ => v.to_string(),                        // 其他类型转换为字符串
//                 }
//             })
//             .collect();
//         // 检查是否有任何一个值包含 search_term
//         let have_search_term = values.iter().any(|i| i.contains(&search_term));
//         if have_search_term {
//             items.push(item);
//             // total_count += childrens.len();
//         }
//     }
//     let start_num = ((page - 1) * per_page) as usize;
//     let mut end_num = (page * per_page) as usize;
//     if end_num > items.len() {
//         end_num = items.len();
//     }

//     // 构造最终的 JSON 响应
//     let json_result;
//     if file.len() > 0 {
//         json_result = json!({
//             "status": 0,
//             "msg": "查询成功",
//             "data": items[0]
//         });
//     } else {
//         json_result = json!({
//             "data": {
//                 "count": items.len(),
//                 "site_count": domains.len(),
//                 "total_count": total_count,
//                 "items": items[start_num..end_num],
//                 "items_count": items[start_num..end_num].len(),
//             },
//             "msg": "查询成功",
//             "status": 0
//         });
//     }
//     return Ok(Json(json_result));
// }

#[derive(Deserialize)]
pub struct CachePutParams {
    domain: String,
    id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CachePutData {
    source: String,
}

// #[axum::debug_handler]
pub async fn cache_update(
    Query(params): Query<CachePutParams>,
    // Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    // Extension(client): Extension<Arc<Minio>>,
    Json(data): Json<CachePutData>,
) -> Result<Response, StatusCode> {
    let domain = params.domain;
    let domain_info = domain_info_from_domain(&domain);
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");

    // 将字符串转换为字节数据
    let file_content = data.source;

    let mut update_data = HashMap::new();
    update_data.insert("source", file_content.as_str());

    let mut conditions: HashMap<&str, &str> = HashMap::new();
    conditions.insert("id", params.id.as_str());

    match pgsql
        .update_data(&table_name, update_data, conditions)
        .await
    {
        Ok(rows) => {
            let json_result = json!({"msg": "缓存源码 编辑成功", "status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        }
        Err(e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // match client.stat_object("cache", &file).await {
    //     Ok(Some(stat)) => {
    //         let content_type = stat.content_type().to_string();
    //         let cache_path_key = KeyArgs::new(&file).content_type(Some(content_type));
    //         // 上传文件到 MinIO
    //         match client
    //             .put_object("cache", cache_path_key, file_content.into())
    //             .await
    //         {
    //             Ok(_) => {
    //                 println!("{} 缓存文件编辑成功", file);
    //                 let json_result = json!({"msg": "缓存源码 编辑成功", "status": 0});
    //                 return Ok(Response::builder()
    //                     .header("Content-Type", "application/json")
    //                     .body(Body::from(json_result.to_string()))
    //                     .unwrap());
    //             }
    //             Err(e) => {
    //                 println!("{} 缓存源码编辑失败: {}", file, e);
    //             }
    //         }
    //     }
    //     Ok(None) | Err(_) => println!("没有缓存文件 {}", file),
    // }
    // return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

#[derive(Deserialize)]
pub struct CacheSourceParams {
    domain: String,
    id: String,
}

pub async fn cache_source(
    Query(params): Query<CacheSourceParams>, // 提取查询参数
    // Extension(client): Extension<Arc<Minio>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
) -> Result<Response, StatusCode> {
    let domain = params.domain;
    let domain_info = domain_info_from_domain(&domain);
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");
    let mut conditions: HashMap<&str, &str> = HashMap::new();
    conditions.insert("id", params.id.as_str());
    // let file = params.file;
    // if let Ok(object_data) = client.get_object("cache", &file).await {
    //     let content = object_data.text().await.unwrap();
    //     let json_result =
    //         json!({"data":{"source":content},"msg": "获取缓存源码 成功", "status": 0});
    //     return Ok(Response::builder()
    //         .header("Content-Type", "application/json")
    //         .body(Body::from(json_result.to_string()))
    //         .unwrap());
    // }
    match pgsql
        .fetch_data(
            &table_name,
            &["source"],
            // HashMap::from([("cache_path", cache_path.as_str())]),
            conditions,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    {
        Ok((rows, total)) => {
            // 将 PgRow 转换为可序列化的格式
            // println!(rows)
            // if total < 1 {
            //     println!("无数据");
            //     return Err(StatusCode::INTERNAL_SERVER_ERROR);
            // }

            let items: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "source": row.get::<String, _>("source"),
                    })
                })
                .collect();

            let content = items[0].clone();

            let json_result = json!({"data":content,"msg": "获取缓存源码 成功", "status": 0});
            return Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(Body::from(json_result.to_string()))
                .unwrap());
        } // 表存在，直接返回数据
        Err(status) => {
            println!("表不存在: {}", status);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    // return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

#[derive(Deserialize)]
pub struct TargetPutParams {
    target_lib: String,
    file: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TargetPutData {
    source: String,
}

// #[axum::debug_handler]
pub async fn target_update(
    Query(params): Query<TargetPutParams>,
    Extension(client): Extension<Arc<Minio>>,
    Json(data): Json<TargetPutData>,
) -> Result<Response, StatusCode> {
    let file = params.file;
    let target_lib = params.target_lib;
    // 将字符串转换为字节数据
    let file_content = data.source.into_bytes();

    match client.stat_object(&target_lib, &file).await {
        Ok(Some(stat)) => {
            let content_type = stat.content_type().to_string();
            let cache_path_key = KeyArgs::new(&file).content_type(Some(content_type));
            // 上传文件到 MinIO
            match client
                .put_object(&target_lib, cache_path_key, file_content.into())
                .await
            {
                Ok(_) => {
                    println!("{} 缓存文件编辑成功", file);
                    let json_result = json!({"msg": "缓存源码 编辑成功", "status": 0});
                    return Ok(Response::builder()
                        .header("Content-Type", "application/json")
                        .body(Body::from(json_result.to_string()))
                        .unwrap());
                }
                Err(e) => {
                    println!("{} 缓存源码编辑失败: {}", file, e);
                }
            }
        }
        Ok(None) | Err(_) => println!("没有缓存文件 {}", file),
    }
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

#[derive(Deserialize)]
pub struct TargetSourceParams {
    target_lib: String,
    file: String,
}

pub async fn target_source(
    Query(params): Query<TargetSourceParams>, // 提取查询参数
    Extension(client): Extension<Arc<Minio>>,
) -> Result<Response, StatusCode> {
    let file = params.file;
    let target_lib = params.target_lib;
    if let Ok(object_data) = client.get_object(&target_lib, &file).await {
        let content = object_data.text().await.unwrap();
        let json_result =
            json!({"data":{"source":content},"msg": "获取缓存源码 成功", "status": 0});
        return Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(json_result.to_string()))
            .unwrap());
    }
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

#[derive(Deserialize)]
pub struct SpiderCountInfoParams {
    days: u32,
}
pub async fn spider_count_info(
    Query(params): Query<SpiderCountInfoParams>, // 提取查询参数
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    // Extension(client): Extension<Arc<Minio>>,
) -> Result<Response, StatusCode> {
    let days = params.days;
    let mut google_spider_datas = [0; 5];
    let mut baidu_spider_datas = [0; 5];
    let mut bing_spider_datas = [0; 5];
    let mut sogou_spider_datas = [0; 5];
    let mut other_spider_datas = [0; 5];
    let mut user_datas = [0; 5];
    for day in 0..days {
        let target_date = Local::now() - Duration::days(day as i64);
        let date_str = target_date.format("%Y-%m-%d").to_string();
        let log_datas = match MyFunc::get_log_datas(&date_str, false, &linecache).await {
            Ok(json_data) => json_data, // 将 json_data 赋值给 log_datas
            Err(err) => {
                eprintln!("Error: {}", err); // 打印错误信息
                return Err(StatusCode::INTERNAL_SERVER_ERROR); // 返回错误状态码
            }
        };
        // 遍历日志数据并统计
        if let Some(logs) = log_datas.as_array() {
            let index = (5 - (day + 1)) as usize;
            for log in logs {
                // 统计不同类型的请求
                if let Some(user_type) = log.get("user_type").and_then(|t| t.as_str()) {
                    match user_type {
                        "谷歌蜘蛛" => google_spider_datas[index] += 1,
                        "百度蜘蛛" => baidu_spider_datas[index] += 1,
                        "搜狗蜘蛛" => sogou_spider_datas[index] += 1,
                        "必应蜘蛛" => bing_spider_datas[index] += 1,
                        "其它蜘蛛" => other_spider_datas[index] += 1,
                        "普通用户" => user_datas[index] += 1,
                        _ => other_spider_datas[index] += 1, // 未知类型归为 "其它蜘蛛"
                    }
                }
            }
        }
    }

    let datetimes = ["4天前", "3天前", "前日", "昨日", "今日"];

    // 获取当前日期
    let today = Local::now().date_naive();

    // 计算各个日期
    let dates = [
        today - Duration::days(4),
        today - Duration::days(3),
        today - Duration::days(2),
        today - Duration::days(1),
        today,
    ];

    // 组合日期和字符串
    let formatted: Vec<String> = datetimes
        .iter()
        .zip(dates.iter())
        .map(|(&s, &date)| format!("{}[{}]", s, date.format("%Y-%m-%d")))
        .collect();
    // 使用 serde_json::Value 来存储混合类型的数据

    let data_json = json!({
        "datetimes":formatted,
        "google_spider_datas":google_spider_datas,
        "baidu_spider_datas":baidu_spider_datas,
        "bing_spider_datas":bing_spider_datas,
        "sogou_spider_datas":sogou_spider_datas,
        "other_spider_datas":other_spider_datas,
        "user_datas":user_datas,
    });

    let json_result = json!({"data":data_json,"msg": "获取蜘蛛数据 成功", "status": 0});
    return Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap());
}

#[derive(Deserialize)]
pub struct QPSInfoParams {
    count: i32,
}
pub async fn qps_info(
    Query(params): Query<QPSInfoParams>, // 提取查询参数
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    // Extension(client): Extension<Arc<Minio>>,
) -> Result<Response, StatusCode> {
    let count = params.count;

    // // 生成 categories（时间序列）
    // let categories = {
    //     let mut res = Vec::new();
    //     let mut now = Local::now();
    //     for _ in 0..10 {
    //         res.push(now.format("%H:%M:%S").to_string()); // 格式化时间为 HH:MM:SS
    //         now = now - chrono::Duration::seconds(2); // 每次减少 2 秒
    //     }
    //     res.reverse(); // 反转数组以保持时间顺序
    //     res
    // };

    // // 生成 categories2（数字序列）
    // let categories2 = (0..10).collect::<Vec<_>>();

    // // 生成 data（随机数据，范围 0-1000）
    // let data = {
    //     // let mut rng = rand::rng();
    //     // rand::rng().random_range(0..vec.len())
    //     (0..10).map(|_| rand::rng().random_range(0..=2000)).collect::<Vec<_>>() // 使用 gen_range 生成随机数
    // };

    // // 生成 spider_datas（随机浮点数，范围 5.0-15.0，保留 1 位小数）
    // // let spider_datas = {
    // //     // let mut rng = rand::rng();
    // //     // (0..10).map(|_| rand::rng().random_range(0..=1000)).collect::<Vec<_>>() // 使用 gen_range 生成随机数
    // //     (0..10).map(|_| ((rand::rng().random_range(5.0..=15.0) * 10.0).round() / 10.0)).collect::<Vec<_>>()
    // // };
    // let spider_datas = {
    //     // let mut rng = rand::rng();
    //     (0..10).map(|_| rand::rng().random_range(0..=1000)).collect::<Vec<_>>() // 使用 gen_range 生成随机数
    //     // (0..10)
    //     //     .map(|_| (rng.random_range(5.0..=15.0) * 10.0) / 10.0) // 生成 5.0-15.0 的随机数，保留 1 位小数
    //     //     .collect::<Vec<_>>()
    // };
    // let log_datas = MyFunc::get_log_datas().await{
    //     Ok(datas){

    //     }
    // };
    // 获取当前时间
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let log_datas = match MyFunc::get_log_datas(&date_str, true, &linecache).await {
        Ok(json_data) => json_data, // 将 json_data 赋值给 log_datas
        Err(err) => {
            eprintln!("Error: {}", err); // 打印错误信息
            return Err(StatusCode::INTERNAL_SERVER_ERROR); // 返回错误状态码
        }
    };

    // 初始化统计数组
    // let mut spider_datas = [0; 6]; // [谷歌蜘蛛, 百度蜘蛛, 搜狗蜘蛛, 必应蜘蛛, 其它蜘蛛, 普通用户]
    let mut spider_datas: [Vec<String>; 6] = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    let mut total_requests = 0; // 总请求数

    // 遍历日志数据并统计
    if let Some(logs) = log_datas.as_array() {
        for log in logs {
            if let Some(timestamp) = log.get("timestamp").and_then(|t| t.as_str()) {
                // 解析时间戳
                if let Ok(log_time) =
                    NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ")
                {
                    let log_time = Local.from_local_datetime(&log_time).unwrap();

                    // 检查日志时间是否在当前时间前 1 到 5 秒内
                    if now - log_time > Duration::seconds(0)
                        && now - log_time <= Duration::seconds(5)
                    {
                        total_requests += 1;

                        // 统计不同类型的请求
                        if let Some(user_type) = log.get("user_type").and_then(|t| t.as_str()) {
                            let ip = log.get("ip").and_then(|u| u.as_str()).unwrap_or("");
                            let url = log
                                .get("request_url")
                                .and_then(|u| u.as_str())
                                .unwrap_or("");
                            let truncated_url = if url.len() > 100 {
                                let mut s = String::with_capacity(103); // 100 chars + 3 for "..."
                                s.push_str(&url[..100]);
                                s.push_str("...");
                                s
                            } else {
                                url.to_string()
                            };
                            let ip_location = my_func.get_country_city(ip).await;
                            let url_str = format!("[{}]{} | {}", ip, ip_location, truncated_url);
                            match user_type {
                                "谷歌蜘蛛" => spider_datas[0].push(url_str),
                                "百度蜘蛛" => spider_datas[1].push(url_str),
                                "搜狗蜘蛛" => spider_datas[2].push(url_str),
                                "必应蜘蛛" => spider_datas[3].push(url_str),
                                "其它蜘蛛" => spider_datas[4].push(url_str),
                                "普通用户" => spider_datas[5].push(url_str),
                                _ => spider_datas[4].push(url_str), // 未知类型归为 "其它蜘蛛"
                            }
                        }
                    }
                }
            }
        }
    }

    // 计算 QPS
    let qps = (total_requests as f64 / 5.0).ceil() as u64;

    // 构建最终的 JSON 数据
    let data_json = json!({
        "qps": qps,
        "spider_data": [
            { "value": spider_datas[0].len(), "name": "谷歌蜘蛛", "urls":spider_datas[0] },
            { "value": spider_datas[1].len(), "name": "百度蜘蛛", "urls":spider_datas[1] },
            { "value": spider_datas[2].len(), "name": "搜狗蜘蛛", "urls":spider_datas[2] },
            { "value": spider_datas[3].len(), "name": "必应蜘蛛", "urls":spider_datas[3] },
            { "value": spider_datas[4].len(), "name": "其它蜘蛛", "urls":spider_datas[4] },
            { "value": spider_datas[5].len(), "name": "普通用户", "urls":spider_datas[5] }
        ],
    });

    let json_result = json!({"data":data_json,"msg": "获取QPS数据 成功", "status": 0});
    return Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json_result.to_string()))
        .unwrap());
    // }
    // return Err(StatusCode::INTERNAL_SERVER_ERROR);
}
