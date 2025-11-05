mod functions;
mod handlers;
mod my_const;
use handlers::api::cache_delete;
use handlers::api::cache_domains;
use handlers::api::cache_query;
use handlers::api::cache_source;
use handlers::api::cache_update;
use handlers::api::config_query;
use handlers::api::config_update;
use handlers::api::file_query;
use handlers::api::file_update;
use handlers::api::login;
use handlers::api::qps_info;
use handlers::api::spider_count_info;
use handlers::api::sql_test;
use handlers::{download_website, robots, sitemap,sitemap_txt, website_index, website_main, website_stream};
use my_const::{CONFIG_FILE_PATH, IPV4BIN, VERSION_URL};
// use handlers::api::line;
use handlers::ad::verify_adhtml;
use handlers::ad::verify_adjs;
use handlers::api::logs;
use handlers::api::program_name;
use handlers::api::replace_query;
use handlers::api::target_delete;
use handlers::api::target_domains;
use handlers::api::target_query;
use handlers::api::target_source;
use handlers::api::target_update;
use handlers::api::version;
use handlers::api::website_create;
use handlers::api::website_delete;
use handlers::api::website_insert;
use handlers::api::website_query;
use handlers::api::website_update;
use handlers::api::WebsiteInsertData;
use handlers::tag::tag_html;

use minio_rsc::{client::ListObjectsArgs, provider::StaticProvider, Minio};
// use regex::Regex;
use sqlx::types::chrono::{DateTime, Utc};
mod middleware;
use crate::functions::func::MyFunc;
use crate::functions::verify::Verify;
// use crate::functions::minio::MinioClient;
// use crate::functions::minio::MinioClientWrapper;
use crate::functions::sql::PgsqlService;
use middleware::middleware;

use async_channel::unbounded;
use axum::{
    extract::Extension,
    http::StatusCode,
    middleware::from_fn,
    response::Redirect,
    routing::{delete, get, post, put, Router},
};
use ip2location_ip2location::bin_format::{Database, TokioFile};
use notify::{self, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
// use std::error::Error;
use std::io::Write;
use std::net::Ipv4Addr;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
    vec,
};

use tldextract_rs::TLDExtract;
// use std::time::Duration;
// use tokio::sync::watch;
// use tokio::time::{sleep, Duration};
use askama::Template;
use cached::proc_macro::cached;
use chrono::Local;
use linecache::AsyncLineCache;
use moka::sync::Cache;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, prelude::*, util::SubscriberInitExt,
};

// 定义模板结构体
#[derive(Template)]
#[template(path = "sitemap.xml")]
struct SitemapTemplate {
    base_url: String,
    urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// 定义元数据结构体
#[derive(Debug, Default)]
pub struct MetaData {
    pub title: Option<String>,
    pub keywords: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    #[serde(rename = "ProgramInfo")]
    program_info: ProgramInfo,

    #[serde(rename = "WebsiteSettings")]
    website_settings: WebsiteSettings,

    #[serde(rename = "SEOFunctions")]
    seo_functions: SEOFunctions,

    #[serde(rename = "AccessPolicy")]
    access_policy: AccessPolicy,

    #[serde(rename = "AdPolicy")]
    ad_policy: AdPolicy,

    #[serde(rename = "GlobalCodeInsertion")]
    global_code_insertion: GlobalCodeInsertion,

    #[serde(rename = "SpiderPolicy")]
    spider_policy: SpiderPolicy,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProgramInfo {
    program_name: String,
    authorization_code: String,
    login_account: String,
    login_password: String,
    amazon_s3_api: String,
    pg_database_url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebsiteSettings {
    auto_site_building: bool,
    auto_https_certificate: bool,
    pan_site_auto_site_building: bool,
    pan_site_crawler_target: bool,
    language: String,
    link_mapping: bool,
    homepage_update_time: u32,
    target_static_save: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SEOFunctions {
    external_filter: Vec<String>,
    external_links: Vec<String>,
    meta_information: bool,
    random_div_attributes: bool,
    random_class_name: bool,
    head_header: String,
    head_footer: String,
    body_header: String,
    body_footer: String,
    html_entities: bool,
    friend_link_count: u32,
    friend_links: Vec<String>,
    seo_404_page: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccessPolicy {
    forced_domain_binding: bool,
    ip_site_referrer: bool,
    pan_site_referrer: bool,
    ua_banlist: Vec<String>,
    ip_banlist: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AdPolicy {
    ad_url: String,
    search_referrer_jump_ad: bool,
    regular_ua_jump_ad: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GlobalCodeInsertion {
    filter_ip: Vec<String>,
    head_header: String,
    head_footer: String,
    body_header: String,
    body_footer: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SpiderPolicy {
    baidu_spider: bool,
    sogou_spider: bool,
    yisou_spider: bool,
    byte_spider: bool,
    bing_spider: bool,
    so_spider: bool,
    google_img_spider: bool,
    google_spider: bool,
    quark_spider: bool,
    yahoo_spider: bool,
    other_spider: bool,
    user: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebsiteInfo {
    #[serde(rename = "domain", default)]
    pub domain: String,
    #[serde(rename = "subdomain", default)]
    pub subdomain: String,
    #[serde(rename = "root_domain", default)]
    pub root_domain: String,
    #[serde(rename = "target", default)]
    pub target: String,
    #[serde(rename = "to_lang", default)]
    pub to_lang: String,
    #[serde(rename = "title", default)]
    pub title: String,
    #[serde(rename = "keywords", default)]
    pub keywords: String,
    #[serde(rename = "description", default)]
    pub description: String,
    #[serde(rename = "link_mapping")]
    pub link_mapping: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReplaceRules {
    #[serde(rename = "replace_mode")]
    pub replace_mode: i32,
    #[serde(rename = "replace_rules_all", default)]
    pub all: Vec<String>,
    #[serde(rename = "replace_rules_index", default)]
    pub index: Vec<String>,
    #[serde(rename = "replace_rules_page", default)]
    pub page: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MuluConfig {
    #[serde(rename = "mulu_tem_max")]
    pub mulu_tem_max: i32,
    #[serde(rename = "mulu_mode", default)]
    pub mulu_mode: String,
    #[serde(rename = "mulu_static")]
    pub mulu_static: bool,
    #[serde(rename = "mulu_template", default)]
    pub mulu_template: Vec<String>,
    #[serde(rename = "mulu_custom_header", default)]
    pub mulu_custom_header: Vec<String>,
    #[serde(rename = "mulu_keywords_file", default)]
    pub mulu_keywords_file: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncludeInfo {
    #[serde(rename = "google_include_info", default)]
    pub google_include_info: Vec<String>,
    #[serde(rename = "bing_include_info", default)]
    pub bing_include_info: Vec<String>,
    #[serde(rename = "baidu_include_info", default)]
    pub baidu_include_info: Vec<String>,
    #[serde(rename = "sogou_include_info", default)]
    pub sogou_include_info: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebsiteConf {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "website_info")]
    pub info: WebsiteInfo,
    #[serde(rename = "replace_rules")]
    pub re: ReplaceRules,
    #[serde(rename = "mulu_config")]
    pub mulu: MuluConfig,
    #[serde(rename = "include_info")]
    pub include: IncludeInfo,
    #[serde(rename = "homepage_update_time")]
    pub homepage_update_time: i32,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetReplaceRules {
    pub all: Vec<String>,
    pub index: Vec<String>,
    pub page: Vec<String>,
}

// -----------------------
#[derive(Debug, Deserialize)]
pub struct WebsiteInfo0 {
    pub target: String,
    pub to_lang: String,
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub link_mapping: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceRules0 {
    pub replace_mode: i32,
    pub all: Vec<String>,
    pub index: Vec<String>,
    pub page: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebsiteConf0 {
    #[serde(rename = "Website-info")]
    pub info: WebsiteInfo,
    #[serde(rename = "Replace-rules")]
    pub re: ReplaceRules,
}

// -----------------------
#[derive(Clone)]
pub struct RequestState {
    scheme: String,
    url: String,
    domain_info: HashMap<String, String>,
    webconfig: WebsiteConf,
    // 其他状态字段...
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    fetching_urls: Cache<String, ()>,
}

// 保存配置文件
async fn save_config(config: &Config) -> Result<(), String> {
    let yaml_data = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(CONFIG_FILE_PATH, yaml_data)
        .await
        .map_err(|e| e.to_string())
}

async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_data = fs::read_to_string(CONFIG_FILE_PATH).await?;
    let config: Config = serde_yaml::from_str(&config_data)?;
    Ok(config)
}

async fn watch_config_changes(
    config_path: String,
    config: Arc<RwLock<Config>>,
) -> notify::Result<()> {
    let (notify_tx, notify_rx) = unbounded();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res| {
            let _ = notify_tx.try_send(res);
        },
        notify::Config::default(),
    )?;
    watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive)?;
    while let Ok(event) = notify_rx.recv().await {
        match event {
            Ok(event) => {
                if let EventKind::Modify(_) = event.kind {
                    println!("Config file changed, reloading...");
                    match load_config().await {
                        Ok(new_config) => {
                            let mut config = match config.write() {
                                Ok(config) => {
                                    println!("{:?}", config);
                                    config
                                }
                                Err(e) => {
                                    println!("Failed to acquire write lock: {:?}", e);
                                    continue;
                                }
                            };
                            *config = new_config;
                            println!("Config reloaded successfully.");
                        }
                        Err(e) => println!("Failed to reload config: {:?}", e),
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}

async fn redirect_to_index() -> (StatusCode, Redirect) {
    (StatusCode::MOVED_PERMANENTLY, Redirect::to("/"))
}

#[cached(
    time = 120, // 设置缓存时间为 60 秒
    key = "String", // 指定缓存键的类型为 String
    convert = r#"{ format!("version") }"#, // 将参数转换为缓存键
)]
pub async fn get_cache_version_text(
    my_func: Arc<MyFunc>,
    version_text: &str,
) -> Result<String, StatusCode> {
    let mut text = version_text.to_string();
    match my_func.fetch_url_to_json(VERSION_URL).await {
        Ok(json_data) => {
            // 提取 JSON 中的 "name" 字段
            if let Some(latest_version) = json_data["name"].as_str() {
                if latest_version.to_string() != text {
                    text.push_str(&format!("🚀 (最新版本:{} 可更新)", latest_version));
                }
            } else {
                text.push_str("🚀 (最新版本:未知 可更新)");
            }
            Ok(text)
        }
        Err(e) => {
            println!("Failed to fetch URL: {}", e);
            Err(e)
        }
    }
}

#[cached(
    key = "String", // 指定缓存键的类型为 String
    convert = r#"{ format!("machine_id") }"#, // 将参数转换为缓存键
    option = true // 只缓存 Some 值
)]
pub async fn get_cache_machine_id() -> Option<String> {
    println!("获取 get_cache_machine_id");
    let verify = Verify::new();
    verify.get_machine_id().await
}

#[cached(
    // size = 10000000, // 设置缓存大小为 10,000,000
    time = 55, // 设置缓存时间为 60 秒
    key = "String", // 指定缓存键的类型为 String
    convert = r#"{ format!("{}:{}", bucket_name, object_name) }"#, // 将参数转换为缓存键
    option = true // 只缓存 Some 值
)]
pub async fn check_object_exists(
    minio_client: Arc<Minio>, // 使用 Arc 共享 Minio 客户端
    bucket_name: &str,
    object_name: &str,
) -> Option<bool> {
    // 检查对象是否存在
    match minio_client.stat_object(bucket_name, object_name).await {
        Ok(Some(_)) => Some(true), // 对象存在，缓存结果
        Ok(None) | Err(_) => None, // 对象不存在或出错，不缓存结果
    }
}

#[cached(
    // size = 10000000, // 设置缓存大小为 10,000,000
    time = 3600, // 设置缓存时间为 60 秒
    key = "String", // 指定缓存键的类型为 String
    convert = r#"{ format!("{}", config_path) }"#, // 将参数转换为缓存键
    option = true // 只缓存 Some 值
)]
pub async fn check_webconfig_is_mapping(
    minio_client: &Arc<Minio>, // 使用 Arc 共享 Minio 客户端
    config_path: &str,
) -> Option<bool> {
    match minio_client.get_object("config", config_path).await {
        Ok(object) => {
            let content = object.text().await.unwrap();
            // println!("content: {}", content);
            // 解析 TOML
            let parsed_config: Result<WebsiteConf, toml::de::Error> = toml::from_str(&content);
            match parsed_config {
                Ok(config) => {
                    // println!("target: {}", config.info.target);
                    Some(config.info.link_mapping)
                }
                Err(e) => {
                    println!("Error parsing TOML: {}", e);
                    None // 解析错误时返回 None
                         // Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(_) => {
            println!("{} 没有配置文件", config_path);
            None
        }
    }
}

#[cached(
    time = 60,
    key = "String",
    convert = r#"{ format!("{}:{}", table_name, page_type) }"#,
    option = true
)]
pub async fn get_cache_urls(
    pgsql: &Arc<PgsqlService>,
    table_name: &str,
    page_type: &str,
) -> Option<Vec<String>> {
    // 从pgsql中获取缓存数据，最多1000条
    let mut conditions = HashMap::new();
    if page_type.len() > 0 {
        conditions.insert("page_type", page_type);
    }

    match pgsql
        .get_random_link(
            table_name,
            &["url"],
            conditions,
            Some(1000), // 获取1000条数据
        )
        .await
    {
        Ok(rows) => {
            // Now handling Vec<PgRow>
            if rows.is_empty() {
                println!("未找到匹配的记录");
                return None;
            }
            // 处理所有URL
            let processed_urls: Vec<String> = rows
                .iter()
                .map(|row| {
                    let url: String = row.get("url");
                    // println!("找到URL: {}", url);
                    url.replace("http://", "//")
                })
                .collect();
            println!("成功处理了 {} 条URL", processed_urls.len());
            Some(processed_urls) // 返回包含所有处理后URL的元组
        }
        Err(err) => {
            println!("从pgsql获取数据时出错: {}", err);
            None // 出错时返回None
        }
    }
}

#[cached(
    time = 60,
    key = "String",
    convert = r#"{ format!("{}:{}:{}", table_name, subdomain, root_domain) }"#,
    option = true
)]
pub async fn get_random_websites(
    pgsql: &Arc<PgsqlService>,
    table_name: &str,
    subdomain: &str,
    root_domain: &str,
) -> Option<Vec<String>> {
    let mut conditions = HashMap::new();
    if !subdomain.is_empty() {
        conditions.insert("subdomain", subdomain);
    }
    if !root_domain.is_empty() {
        conditions.insert("root_domain", subdomain);
    }

    match pgsql
        .get_random_domain(table_name, &["domain"], conditions, Some(100))
        .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                println!("未找到匹配的记录");
                return None;
            }

            let mut result = Vec::new();
            for row in rows {
                let domain: String = row.get("domain");
                // let title: String = row.get("title");
                // let keywords: String = row.get("keywords");
                result.push(domain);
            }
            println!("成功处理了 {} 条记录", result.len());
            Some(result)
        }
        Err(err) => {
            println!("从pgsql获取数据时出错: {}", err);
            None
        }
    }
}

#[cached(
    time = 3600,
    key = "String",
    convert = r#"{ format!("{}:{}", bucket_name, object_name) }"#,
    option = true
)]
pub async fn get_object_domains(
    minio_client: &Arc<Minio>,
    bucket_name: &str,
    object_name: &str,
) -> Option<(Vec<String>, Vec<String>)> {
    let mut www = Vec::new(); // 存储包含"www"的域名
    let mut other = Vec::new(); // 存储其他域名
    let mut continuation_token: Option<String> = None; // 分页用的延续令牌

    if object_name == "/" {
        // 处理目录列表情况
        loop {
            let mut args = ListObjectsArgs::default()
                .max_keys(1000) // 每次最多返回1000个对象
                .delimiter("/"); // 使用斜杠作为分隔符来获取子目录

            if let Some(token) = &continuation_token {
                args = args.continuation_token(token); // 设置分页令牌
            }

            let result = match minio_client.list_objects(bucket_name, args).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("列出桶 '{}' 中的对象失败: {}", bucket_name, e);
                    return None;
                }
            };

            // 处理每个子目录的前缀
            for prefix in result.common_prefixes {
                let mut sub_continuation_token: Option<String> = None; // 子目录的分页令牌

                // 循环列出该子目录下的所有对象
                loop {
                    let mut sub_args = ListObjectsArgs::default()
                        .prefix(&prefix.prefix) // 设置子目录前缀
                        .max_keys(1000); // 每次最多返回1000个对象

                    if let Some(token) = &sub_continuation_token {
                        sub_args = sub_args.continuation_token(token); // 设置子目录分页令牌
                    }

                    let sub_result = match minio_client.list_objects(bucket_name, sub_args).await {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!(
                                "列出桶 '{}' 中前缀为 '{}' 的对象失败: {}",
                                bucket_name, prefix.prefix, e
                            );
                            break; // 出错时跳出子目录循环，继续处理下一个前缀
                        }
                    };

                    // 将子目录中的对象分为www域名和其他域名
                    for content in sub_result.contents {
                        let parts: Vec<&str> = content.key.split('/').collect();
                        if parts.len() >= 2 {
                            let root_domain = parts[0].to_string(); // 提取根域名，例如"domain11014.com"
                            let full_domain_part = parts[1].trim_end_matches(".toml"); // 去掉".toml"后缀
                            if full_domain_part.to_string().trim_start_matches("www.")
                                == root_domain
                            {
                                www.push(full_domain_part.to_string()); // 添加完整域名到www列表
                            } else {
                                other.push(full_domain_part.to_string()); // 添加根域名到other列表
                            }
                        } else {
                            eprintln!("无效的key格式: {}", content.key);
                        }
                    }

                    if !sub_result.is_truncated {
                        // 如果子目录没有更多数据
                        break;
                    }
                    sub_continuation_token = Some(sub_result.next_continuation_token);
                    // 更新子目录分页令牌
                }
            }

            if !result.is_truncated {
                // 如果主目录没有更多数据
                break;
            }
            continuation_token = Some(result.next_continuation_token); // 更新主分页令牌
        }
    } else {
        // 处理文件列表情况
        loop {
            let mut args = ListObjectsArgs::default()
                .prefix(object_name) // 设置查询前缀
                .max_keys(1000); // 每次最多返回1000个对象

            if let Some(token) = &continuation_token {
                args = args.continuation_token(token); // 设置分页令牌
            }

            let result = match minio_client.list_objects(bucket_name, args).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "列出桶 '{}' 中前缀为 '{}' 的对象失败: {}",
                        bucket_name, object_name, e
                    );
                    return None;
                }
            };

            // 将对象分为www域名和其他域名
            for content in result.contents {
                let parts: Vec<&str> = content.key.split('/').collect();
                if parts.len() >= 2 {
                    let root_domain = parts[0].to_string(); // 提取根域名，例如"domain11014.com"
                    let full_domain_part = parts[1].trim_end_matches(".toml"); // 去掉".toml"后缀
                    if full_domain_part.to_string().trim_start_matches("www.") == root_domain {
                        www.push(full_domain_part.to_string()); // 添加完整域名到www列表
                    } else {
                        other.push(full_domain_part.to_string()); // 添加根域名到other列表
                    }
                } else {
                    eprintln!("无效的key格式: {}", content.key);
                }
            }

            if !result.is_truncated {
                // 如果没有更多数据
                break;
            }
            continuation_token = Some(result.next_continuation_token); // 更新分页令牌
        }
    }

    if www.is_empty() && other.is_empty() {
        eprintln!(
            "在桶 '{}' 中前缀为 '{}' 未找到任何对象",
            bucket_name, object_name
        );
        None
    } else {
        println!(
            "在桶 '{}' 中前缀为 '{}' 找到 {} 个www域名和 {} 个其他域名",
            bucket_name,
            object_name,
            www.len(),
            other.len()
        );
        Some((www, other)) // 返回包含两个列表的元组
    }
}

#[cached(
    // size = 10000000, // 设置缓存大小为 10,000,000
    // time = 60, // 设置缓存时间为 60 秒
    key = "String", // 指定缓存键的类型为 String
    convert = r#"{ format!("{}", domain) }"#, // 将参数转换为缓存键
    // option = true // 只缓存 Some 值
)]
pub fn domain_info_from_domain(domain: &str) -> HashMap<String, String> {
    // println!("get_domain_info domain: {}", domain);
    let source = tldextract_rs::Source::Snapshot;
    let suffix = tldextract_rs::SuffixList::new(source, false, None); // 不需要 unwrap
    let mut extract = TLDExtract::new(suffix, true).unwrap(); // 假设这里返回的是 Result 类型
    let result = extract.extract(domain);

    match result {
        Ok(data) => {
            let subdomain = data.subdomain.unwrap_or_else(|| "".to_string());
            let root_domain = data.registered_domain.unwrap_or_else(|| "".to_string());
            let full_domain = format!("{}.{}", subdomain, root_domain);

            let mut map = HashMap::new();
            map.insert("subdomain".to_string(), subdomain);
            map.insert("full_domain".to_string(), full_domain);
            map.insert("root_domain".to_string(), root_domain);

            map
        }
        Err(e) => {
            // 处理错误，例如打印错误消息或返回默认值
            eprintln!("{} Error extracting domain: {}", domain, e);
            HashMap::new()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置日志文件
    let info_file_appender = RollingFileAppender::new(Rotation::DAILY, "log", "app.log");
    let (info_non_blocking, _info_guard) = tracing_appender::non_blocking(info_file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::Layer::default()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_level(true)
                .with_target(true)
                .with_filter(LevelFilter::INFO),
        ) // 配置控制台输出
        .with(
            fmt::Layer::default()
                .with_writer(info_non_blocking)
                .with_ansi(false)
                .with_level(false)
                .with_target(false)
                .with_filter(LevelFilter::INFO),
        ) // 配置日志文件输出
        .init();

    // 加载配置文件
    let config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("配置文件加载失败: {:?}", e);
            return Err(e);
        }
    };
    let config = Arc::new(RwLock::new(config));
    // 启动配置文件监视
    tokio::spawn(watch_config_changes(
        CONFIG_FILE_PATH.to_string(),
        Arc::clone(&config),
    ));

    // ips
    let ips = MyFunc::get_ips();
    println!("服务器所有IP:{:?}", ips);

    println!(
        "Amazon_S3_API: {}",
        config.read().unwrap().program_info.amazon_s3_api
    );

    let minio_info = MyFunc::parse_minio_addr(&config.read().unwrap().program_info.amazon_s3_api);
    let provider = StaticProvider::new(&minio_info["username"], &minio_info["password"], None);

    let client = Minio::builder()
        .endpoint(&minio_info["address"])
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();

    // let minio_client = MinioClient::new(
    //     &minio_info["address"],
    //     &minio_info["username"],
    //     &minio_info["password"],
    //     false,
    // );

    let mut give_free_authorization_code = true;
    // 检查并创建 Buckets
    for bucket in [
        "replace",
        "target-zh",
        "target-en2zh",
        "target-en",
        "target-zh2en",
    ] {
        let exists = client.bucket_exists(bucket).await?;
        if !exists {
            println!("Bucket '{}' does not exist. Creating...", bucket);
            client.make_bucket(bucket, true).await?;

            println!("Bucket '{}' created successfully.", bucket);
        } else {
            println!("Bucket '{}' already exists.", bucket);
            give_free_authorization_code = false;
        }
    }

    // 检查并创建文件
    let mut files: HashMap<&str, &str> = HashMap::new();
    files.insert(
        "doc/keywords/keyword.txt",
        "MirrorElf镜像精灵\nMirrorElf\n镜像精灵",
    );
    files.insert(
        "doc/keywords/keyword2.txt",
        "MirrorElf镜像精灵\nMirrorElf\n镜像精灵",
    );
    files.insert("doc/names/中文人名.txt", "张三\n李四\n王五\n赵六");
    files.insert(
        "doc/names/person_name.txt",
        "zhangsan\nlisi\nwangwu\nzhaoliu",
    );
    files.insert("doc/404_links.txt", "https://www.bing.com/search?q={@keyword} {@ad}&qs=HS&sc=20-0&cvid=【uuid】&FORM=QBLH&sp=1&lq=0");
    files.insert(
        "doc/大小写字母.txt",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    );
    files.insert("doc/ad.txt", "mirrorelf.com");
    files.insert("doc/article.txt", "「镜像精灵」是一款专业的网站镜像工具，可快速创建实时同步的站点副本，支持数据备份、负载均衡和全球加速。通过智能同步引擎和分布式部署，帮助企业和开发者轻松实现业务连续性保障和访问优化。\n「镜像精灵」让网站镜像从未如此简单！一键克隆完整网站，支持定时同步、多节点部署和访问控制。无论是内容备份、开发测试还是全球加速，都能提供高效稳定的解决方案，助您打造永不掉线的在线服务。");
    files.insert("doc/bind_domain.txt", "mirrorelf.com");
    files.insert("doc/emoji.txt", "📱📲☎📞📟📠🔋🔌💻💽💾💿📀🎥📺📷📹📼🔍");
    files.insert("doc/friend_link.txt", "https://www.mirrorelf.com");
    files.insert("doc/keyword.txt", "MirrorElf镜像精灵\nMirrorElf\n镜像精灵");
    files.insert(
        "doc/push_link.txt",
        "https://www.facebook.com/watch/explore/{@keyword} {@ad}",
    );
    files.insert("doc/sign.txt", "!@#$%^&*");
    files.insert("doc/target_en.txt", "zh|www.mirrorelf.com");
    files.insert("doc/target_zh.txt", "en|www.mirrorelf.com");
    files.insert("doc/website.txt", "www.domain.com___zh|www.mirrorelf.com___网站标题___网站关键词___网站描述___关于我们----------{keyword}##########公司名称----------【关键词】___关于我们 -> {keyword} ; 公司名称 -> 【关键词】");
    files.insert("templates/seo404.html", r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<script>
document.write('<meta name="viewport" content="width=device-width, initial-scale=1.0">');
document.write('<style>html,body {width:100%;height:100%;overflow:hidden;margin:0;padding:0;}</style>');
document.write(`
<div style="width:100%;height:100%;position:fixed;top:0;left:0;z-index:2147483647;">
    <iframe src="{@404_links#996}" style="width:100%;height:100%;border:none;"></iframe>
</div>
`);
</script>
<title>{@404_links#996}</title>
</head>
<body>
</body>
</html>"#);
    for (filepath, content) in files {
        let path = Path::new(filepath);

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 检查文件是否存在
        if !path.exists() {
            println!("File '{}' does not exist. Creating...", filepath);
            // 创建并写入文件
            let mut file = std::fs::File::create(filepath)?;
            file.write_all(content.as_bytes())?;
            println!("File '{}' created successfully.", filepath);
        } else {
            // println!("File '{}' already exists.", filepath);
        }
    }

    // 加载数据库配置
    let database_url = config.read().unwrap().program_info.pg_database_url.clone();
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .min_connections(10)
        .connect(&database_url)
        .await?;
    let pgsql = PgsqlService::new(pool);
    match pgsql.check_db_version().await {
        Ok((version_is_ok, version_info)) => {
            if version_is_ok {
                println!("数据库版本: {}", version_info);
            } else {
                panic!("pgsql数据库版本过低，程序退出。");
            }
        }
        Err(_e) => {
            panic!("pgsql数据库版本检查失败，程序退出。");
        }
    };

    // 加载数据库文件
    let ipdb = Database::<TokioFile>::new(IPV4BIN, 2).await?;

    let my_func = MyFunc::new(ips, ipdb);
    let verify = Verify::new();

    // 新服务器 自动申请免费授权码 判断config授权码为888
    if give_free_authorization_code {
        let authorization_code = config
            .read()
            .unwrap()
            .program_info
            .authorization_code
            .clone();
        if authorization_code == "888" {
            // 给免费授权
            match verify.encrypt_data("1").await {
                Some(free_code) => {
                    // 更新配置
                    let mut config_data = config.write().unwrap();
                    config_data.program_info.authorization_code = free_code.clone();
                    // 保存配置
                    match save_config(&config_data).await {
                        Ok(_) => {
                            println!("免费授权码更新成功: {}", free_code);
                        }
                        Err(e) => {
                            println!("免费授权失败, 保存配置时出错: {}", e);
                        }
                    }
                }
                None => {
                    println!("免费授权失败，无法生成授权码");
                }
            }
        }
    }

    let paths_to_redirect = vec![
        "/index.html",
        "/index.php",
        "/index.asp",
        "/index.jsp",
        "/index.htm",
        "/index.shtml",
        "/index",
        "/home.html",
        "/xedni.html",
        "/xedni.php",
        "/xedni.asp",
        "/xedni.jsp",
        "/xedni.htm",
        "/xedni.shtml",
        "/xedni",
        "/emoh.html",
        "/indexPer",
    ];

    let linecache = AsyncLineCache::new();

    let middleware_stack = ServiceBuilder::new()
        // 添加请求和响应的高级跟踪
        // .layer(TraceLayer::new_for_http())
        // 添加响应压缩
        .layer(CompressionLayer::new())
        // 将ServiceBuilder转换为tower::Layer
        .into_inner();

    // 创建一个广播通道
    let (tx, _) = broadcast::channel(16);
    // 创建一个带有过期时间的缓存
    let fetching_urls: Cache<String, ()> = Cache::builder()
        .time_to_live(Duration::from_secs(30)) // 设置缓存项的过期时间为10秒
        .build();
    let app_state = AppState {
        tx: tx.clone(),
        fetching_urls,
    };

    // // 模拟日志生成
    // tokio::spawn(async move {
    //     let mut interval = tokio::time::interval(Duration::from_secs(1));
    //     loop {
    //         interval.tick().await;
    //         let log_entry = format!("data: Log entry at {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S"));
    //         let _ = tx.send(log_entry); // 发送日志到通道
    //     }
    // });
    tokio::spawn(async move {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let file_path = format!("log/app.log.{}", today);

        // 打开日志文件
        let file = match File::open(&file_path).await {
            Ok(file) => file,
            Err(_) => {
                eprintln!("Failed to open log file: {}", file_path);
                return;
            }
        };

        // 使用 BufReader 逐行读取文件
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // 记录当前文件大小
        let mut last_size = match tokio::fs::metadata(&file_path).await {
            Ok(metadata) => metadata.len(),
            Err(_) => {
                eprintln!("Failed to get file metadata: {}", file_path);
                return;
            }
        };

        // 定期检查文件是否有新内容
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            // 检查文件大小是否变化
            let current_size = match tokio::fs::metadata(&file_path).await {
                Ok(metadata) => metadata.len(),
                Err(_) => {
                    eprintln!("Failed to get file metadata: {}", file_path);
                    continue;
                }
            };

            // 如果文件大小变化，读取新增内容
            if current_size > last_size {
                let mut new_content = Vec::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    new_content.push(line);
                }

                last_size = current_size;
                if !new_content.is_empty() {
                    for line in new_content {
                        let _ = tx.send(format!("{}\n", line));
                    }
                }
            }
        }
    });

    // 配置路由
    let mut app = Router::new()
        // .with_state(app_state)
        .route("/", get(website_index))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/sitemap.txt", get(sitemap_txt))
        .route("/_static/ad.js", get(verify_adjs))
        .route("/_static/ad.html", get(verify_adhtml))
        .route("/_tag.html", get(tag_html))
        .route_service("/_/admin", ServeFile::new("_/admin/index.html")) // 静态文件服务
        .route_service("/_/login", ServeFile::new("_/admin/login.html")) // 静态文件服务
        .route_service("/favicon.ico", ServeFile::new("_/static/favicon.ico")) // 静态文件服务
        .nest_service("/_", ServeDir::new("_")) // 静态文件目录服务
        .route("/-/*url", get(website_stream)) // 处理流式网站请求
        .route("/--/*url", get(download_website)) // 处理流式网站请求
        // .route("/@/*url", get(minio_stream)) // 处理流式网站请求
        .route("/*url", get(website_main)) // 处理流式网站请求
        // .route("/_api/version", get(machineid))
        .route("/_api/version", get(version))
        .route("/_api/login", post(login))
        .route("/_api/program_name", get(program_name))
        .route("/_api/sql", get(sql_test))
        .route("/_api_/logs", get(logs))
        // .route("/_api_/line", get(line))
        .route("/_api_/config", get(config_query))
        .route("/_api_/config", put(config_update))
        .route("/_api_/cache/domains", get(cache_domains))
        .route("/_api_/cache/query", get(cache_query))
        .route("/_api_/cache/update", put(cache_update))
        .route("/_api_/cache/source", get(cache_source))
        .route("/_api_/cache/delete", delete(cache_delete))
        .route("/_api_/website/query", get(website_query))
        .route("/_api_/replace/query", get(replace_query))
        .route("/_api_/website/insert", post(website_insert))
        .route("/_api_/website/create", post(website_create))
        .route("/_api_/website/delete", delete(website_delete))
        .route("/_api_/website/update", put(website_update))
        .route("/_api_/file/query", get(file_query))
        .route("/_api_/file/update", put(file_update))
        .route("/_api_/target/query", get(target_query))
        .route("/_api_/target/domains", get(target_domains))
        .route("/_api_/target/delete", delete(target_delete))
        .route("/_api_/target/update", put(target_update))
        .route("/_api_/target/source", get(target_source))
        .route("/_api_/info/spider_count", get(spider_count_info))
        .route("/_api_/info/qps", get(qps_info))
        .layer(Extension(app_state))
        .layer(from_fn(middleware)) // 添加中间件层
        .layer(Extension(config.clone())) // 将配置添加为扩展
        .layer(Extension(Arc::new(linecache))) // 将配置添加为扩展
        .layer(Extension(Arc::new(pgsql))) // 将配置添加为扩展
        // .layer(Extension(Arc::new(PgsqlService { pool }))) // 将数据库连接池实例化后添加为扩展
        .layer(Extension(Arc::new(my_func)))
        .layer(Extension(Arc::new(verify)))
        .layer(Extension(Arc::new(client)))
        // .layer(Extension(Arc::new(minio_client)))
        .layer(middleware_stack);
    //

    for path in paths_to_redirect {
        app = app.route(path, get(redirect_to_index));
    }

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:16888")
        .await
        .unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
