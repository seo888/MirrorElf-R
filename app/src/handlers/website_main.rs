use crate::check_object_exists;
use crate::domain_info_from_domain;
use crate::functions::ai::AiTrans;
use crate::functions::func::MyFunc;
use crate::functions::sql::PgsqlService;
use crate::get_random_websites;
use crate::handlers::website_stream;
use crate::my_const::CACHE_PAGE_SUFFIX;
use crate::my_const::TIMESTAMP_REGEX;
use crate::my_const::{
    BODY_FOOTER_REGEX, BODY_HEADER_REGEX, CONFIG_FILE_PATH, REPALCE_CONTENT, SECRET, TITLE_REGEX,
    VERSION,
};
use crate::website_insert;
use crate::AppState;
use crate::Config;
use crate::RequestState;
// use crate::Seo404Template;
use crate::WebsiteInsertData;
// use crate::MetaData;
use crate::TargetReplaceRules;
use crate::WebsiteConf;
use crate::WebsiteConf0;
use askama::Template;
use axum::{
    body::Body,
    // extract::State,
    extract::{Json, Path, Request},
    http::header::USER_AGENT,
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use linecache::AsyncLineCache;
use minio_rsc::{client::KeyArgs, Minio};
use rand::prelude::IndexedRandom;
use rand::seq::IteratorRandom; // <-- This import is required for .choose()
use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::{
    // collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};
use tokio::fs::read_to_string;
use tokio::spawn;
use toml;
use uuid::Uuid;

// use toml::de::Error as TomlError;
// use tracing::info; // 导入 Config 结构体

// 处理根路径的函数
pub async fn website_index(
    Extension(state): Extension<AppState>,
    Extension(config): Extension<Arc<RwLock<Config>>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    req: Request,
) -> Result<Response, StatusCode> {
    // 调用 website_main 来处理根路径
    website_main(
        Path("".to_string()),
        Extension(state),
        Extension(config), // 使用 Extension 包装 config
        Extension(pgsql),  // 使用 Extension 包装 pgsql
        Extension(my_func),
        Extension(client),
        Extension(linecache),
        req,
    )
    .await
}

pub async fn website_main(
    Path(path): Path<String>,
    // State(state): State<AppState>,
    Extension(state): Extension<AppState>,
    Extension(config): Extension<Arc<RwLock<Config>>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(client): Extension<Arc<Minio>>,
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    req: Request,
) -> Result<Response, StatusCode> {
    // if path.starts_with("google") && path.ends_with(".html") && path.len() == 27 {
    //     return Ok(Response::builder()
    //         .status(200)
    //         .header("Content-Type", "text/html")
    //         .body(Body::from(format!("google-site-verification: {}", path)))
    //         .unwrap());
    // }
    // println!("path: {}", path);
    // let req_state = req.extensions().get::<RequestState>().unwrap();
    let req_state = {
        req.extensions().get::<RequestState>().unwrap().clone()
    };
    let path_lower = path.to_lowercase();

    // 获取完整的 URI
    let mut uri = req.uri().to_string();

    // 获取 scheme (http 或 https)
    // let scheme = uri.scheme_str().unwrap_or("http");

    // // 获取主机名
    // let host = req.headers().get("host").unwrap().to_str().unwrap();

    // // 提取域名（去除端口号，转为小写）
    // let domain = host.split(":").next().unwrap();
    // let mut domain = if domain == "localhost" {
    //     "www.localhost.com".to_string()
    // } else {
    //     domain.to_string()
    // };
    // domain.make_ascii_lowercase();

    let domain_info = &req_state.domain_info;
    let domain = domain_info["full_domain"].clone();
    // let domain_info = domain_info_from_domain(&domain);

    let mut come_from_url = format!("http://{domain}{uri}");
    let full_url = format!("{}{}", domain, uri);
    let cache_path = format!("{}", full_url.trim_end_matches('/'));
    let mut cache_path = MyFunc::path_clean(&cache_path);
    let table_name = format!(
        "{}__{}",
        domain_info["subdomain"], domain_info["root_domain"]
    )
    .replace(".", "_");

    // 获取 config_dict
    let config_dict = config.read().unwrap().clone();
    let path_suffix = if path_lower.contains(".") {
        format!(".{}", path_lower.split('.').last().unwrap_or(""))
    } else {
        ".".to_string()
    };
    // let cache_page_suffix = [".php", ".asp", ".jsp", ".html", ".htm", ".shtml"];

    // println!("path_suffix：{}",path_suffix);
    // 如果命中要缓存的资源后缀名，则查询缓存
    if path_suffix == "."
        || CACHE_PAGE_SUFFIX.contains(&path_suffix.as_str())
        || config_dict
            .website_settings
            .target_static_save
            .contains(&path_suffix.as_str())
    {
        // --------------- 【缓存处理】 ---------------

        // println!("{} 开始查找缓存文件", cache_path);
        // let columns = &["page_type","uri","source"];  // 要查询的列
        // let conditions = HashMap::from([("cache_path", cache_path.as_str())]); // WHERE 条件

        // 记录开始时间
        let clean_start_time = Instant::now();

        // 从pgsql中获取缓存数据
        let cache_rows = pgsql
            .fetch_or_create_table(
                &table_name,
                &["page_type", "uri", "source", "target"],
                HashMap::from([("cache_path", cache_path.as_str())]),
                Some(1),
            )
            .await?;

        // 记录结束时间
        let clean_end_time = Instant::now();

        // 计算耗时（以毫秒为单位）
        let clean_duration_ms = (clean_end_time - clean_start_time).as_millis();
        println!(
            "{} 存在缓存：{} 查询数据库耗时：{} ms",
            cache_path,
            !cache_rows.is_empty(),
            clean_duration_ms
        );

        // println!("cache_rows: {:?}",cache_rows);
        if !cache_rows.is_empty() {
            let cache_row = &cache_rows[0];
            let page_type: String = cache_row.get("page_type");
            let source: String = cache_row.get("source");
            let cache_uri: String = cache_row.get("uri");
            let target_minio_path: String = cache_row.get("target");
            match page_type.as_str() {
                "缓存" => {
                    println!("{} 发现缓存文件，直接返回", cache_path);
                    return Ok(Response::builder()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .header("M-Page-Mode", "cache-html")
                        .body(source.into())
                        .unwrap());
                }
                "映射" => {
                    println!("{} 发现映射文件", cache_path);
                    // 获取映射的真实缓存文件路径
                    let cache_url = format!("{}{}", domain, cache_uri);
                    let real_cache_path = MyFunc::path_clean(&cache_url);
                    // 从pgsql中获取缓存数据
                    let mapping_cache_rows = pgsql
                        .fetch_or_create_table(
                            &table_name,
                            &["page_type", "uri", "source"],
                            HashMap::from([("cache_path", real_cache_path.as_str())]),
                            Some(1),
                        )
                        .await?;
                    if !mapping_cache_rows.is_empty() {
                        let mapping_cache_row = &mapping_cache_rows[0];
                        let source: String = mapping_cache_row.get("source");
                        // 存在映射的真实缓存文件
                        return Ok(Response::builder()
                            .header("Content-Type", "text/html; charset=utf-8")
                            .header("M-Page-Mode", "cache-html")
                            .body(source.into())
                            .unwrap());
                    } else {
                        println!(
                            "{} 映射的缓存文件:{} 不存在 改当前uri为:{}",
                            cache_path, real_cache_path, &cache_uri
                        );
                        // 这里就相当于 正常的访问进来了
                        cache_path = real_cache_path.to_string();
                        uri = cache_uri.to_string();
                        come_from_url = format!("http://{domain}{uri}");
                    }
                }
                "静态" => {
                    println!("{} 发现目标静态资源文件", cache_path);
                    if let Some((target_bucket, target_path)) = target_minio_path.split_once("://")
                    {
                        // println!("Target Bucket: {}", target_bucket);
                        // println!("Target Path: {}", target_path);
                        // let target_path = MyFunc::path_clean(target_path.trim_end_matches("/"));
                        match client.stat_object(target_bucket, target_path).await {
                            Ok(Some(stat)) => {
                                // 获取目标资源的 content_type
                                let content_type;
                                let t_url;
                                if let Some((ct, url)) = stat.content_type().split_once("|") {
                                    content_type = ct.to_string();
                                    t_url = url.to_string();
                                    println!("Content-Type: {}", content_type);
                                    println!("URL: {}", t_url);
                                } else {
                                    content_type = stat.content_type().to_string(); // 使用完整的 content_type
                                    t_url = "".to_string(); // 或者设置为默认 URL
                                };
                                match client.get_object(target_bucket, target_path).await {
                                    Ok(object) => {
                                        let body = Body::from_stream(object.bytes_stream());
                                        return Ok(Response::builder()
                                            .header("Content-Type", content_type)
                                            .header("M-Page-Mode", "cache-static")
                                            .header("Cache-Control", "public, max-age=604800") // 设置浏览器缓存
                                            .body(body)
                                            .unwrap());
                                    }
                                    Err(_) => return Err(StatusCode::NOT_FOUND),
                                }
                            }
                            Ok(None) | Err(_) => return Err(StatusCode::NOT_FOUND),
                        }
                    } else {
                        println!("target_minio_path does not contain '://'");
                    }
                }
                _ => {
                    println!("未知类型: {}", page_type);
                    // 预留给目录
                    return Err(StatusCode::NOT_FOUND);
                }
            }
        }
    }

    // else
    // {
    //     println!("{} 没有缓存文件", cache_path)
    // }

    // 从minio中获取缓存数据
    // match client.stat_object("cache", &cache_path).await {
    //     Ok(Some(stat)) => {
    //         let content_type = stat.content_type();
    //         if content_type.starts_with("uri:") {
    //             let internal_uri = content_type.trim_start_matches("uri:");
    //             let internal_uri = if let Some((uri_part, _)) = internal_uri.split_once("|") {
    //                 uri_part
    //             } else {
    //                 internal_uri
    //             };
    //             let internal_url = format!("{}{}", domain, internal_uri);
    //             let real_cache_path = MyFunc::path_clean(&internal_url);

    //             match client.get_object("cache", &real_cache_path).await {
    //                 Ok(object) => {
    //                     let body = Body::from_stream(object.bytes_stream());
    //                     return Ok(Response::builder()
    //                         .header("Content-Type", "text/html; charset=utf-8")
    //                         .header("M-Page-Mode", "cache-html-link")
    //                         .body(body)
    //                         .unwrap());
    //                 }
    //                 Err(_) => {
    //                     println!(
    //                         "{} 映射的缓存文件:{} 不存在 改当前uri为:{}",
    //                         cache_path, real_cache_path, internal_uri
    //                     );
    //                     cache_path = real_cache_path.to_string();
    //                     uri = internal_uri.to_string();
    //                 }
    //             }
    //         } else if content_type.starts_with("target-") {
    //             if let Some((target_bucket, target_path)) = content_type.split_once("://") {
    //                 println!("Target Bucket: {}", target_bucket);
    //                 println!("Target Path: {}", target_path);
    //                 // let target_path = MyFunc::path_clean(target_path.trim_end_matches("/"));
    //                 match client.stat_object(target_bucket, target_path).await {
    //                     Ok(Some(stat)) => {
    //                         // let content_type = stat.content_type();
    //                         let content_type;
    //                         let t_url;
    //                         if let Some((ct, url)) = stat.content_type().split_once("|") {
    //                             content_type = ct.to_string();
    //                             t_url = url.to_string();
    //                             println!("Content-Type: {}", content_type);
    //                             println!("URL: {}", t_url);
    //                         } else {
    //                             content_type = stat.content_type().to_string(); // 使用完整的 content_type
    //                             t_url = "".to_string(); // 或者设置为默认 URL
    //                         };
    //                         match client.get_object(target_bucket, target_path).await {
    //                             Ok(object) => {
    //                                 let body = Body::from_stream(object.bytes_stream());
    //                                 return Ok(Response::builder()
    //                                     .header("Content-Type", content_type)
    //                                     .header("M-Page-Mode", "cache-static-link")
    //                                     .body(body)
    //                                     .unwrap());
    //                             }
    //                             Err(_) => return Err(StatusCode::NOT_FOUND),
    //                         }
    //                     }
    //                     Ok(None) | Err(_) => return Err(StatusCode::NOT_FOUND),
    //                 }
    //             } else {
    //                 println!("Content type does not contain '://'");
    //             }
    //         } else {
    //             match client.get_object("cache", &cache_path).await {
    //                 Ok(object) => {
    //                     let body = Body::from_stream(object.bytes_stream());
    //                     return Ok(Response::builder()
    //                         .header("Content-Type", "text/html; charset=utf-8")
    //                         .header("M-Page-Mode", "cache-html")
    //                         .body(body)
    //                         .unwrap());
    //                 }
    //                 Err(_) => return Err(StatusCode::NOT_FOUND),
    //             }
    //         }
    //     }
    //     Ok(None) | Err(_) => {
    //         // println!("{} 没有缓存文件", cache_path)
    //     }
    // }

    // --------------- 【获取网站配置】 ---------------

    let config_path = format!(
        "{}/{}.toml",
        domain_info["root_domain"], domain_info["full_domain"]
    );

    // 调用单独的函数处理获取或生成配置文件的逻辑
    // let mut webconfig = my_func
    //     .fetch_or_create_config(
    //         domain_info["subdomain"].as_str() == "www",
    //         &config_dict,
    //         &pgsql,
    //         &config_path,
    //         &domain_info["full_domain"],
    //     )
    //     .await?;

    let mut webconfig = req_state.webconfig.clone();

    // 处理TDK转码
    if config_dict.seo_functions.html_entities {
        webconfig.info.title = MyFunc::encode_to_html_entities(&webconfig.info.title);
        webconfig.info.description = MyFunc::encode_to_html_entities(&webconfig.info.description);
        webconfig.info.keywords = MyFunc::encode_to_html_entities(&webconfig.info.keywords);
    }

    // --------------- 【获取目标资源】 ---------------
    // let (target_lang, target_domain) = if webconfig.info.target.contains('|') {
    //     let mut parts = webconfig.info.target.split('|');
    //     let lang = parts.next().unwrap_or(""); // 获取语言部分
    //     let domain = parts.next().unwrap_or(""); // 获取域名部分
    //     (lang, domain)
    // } else {
    //     // 如果没有 |，则默认语言为空，域名是 target_domain
    //     ("zh", webconfig.info.target.as_str())
    // };
    let (target_lang, target_domain) =
        MyFunc::get_target_lang_domain(webconfig.info.target.as_str());

    let target_real_url = format!("http://{target_domain}{uri}");

    // 判断不在目标资源存储列表内 则 流式响应
    if path_lower.contains(".")
        && !CACHE_PAGE_SUFFIX.contains(&path_suffix.as_str())
        && !config_dict
            .website_settings
            .target_static_save
            .contains(&path_suffix.as_str())
    {
        println!("{} 目标静态资源 流式响应", target_real_url);
        return website_stream(Path(target_real_url), Extension(my_func)).await;
    }

    // if path_lower.contains('.') && !pass_list.iter().any(|ext| path_lower.contains(ext)) {
    //     if path_lower.split('.').last().map(|ext| format!(".{}", ext))
    // if !config_dict
    //     .website_settings
    //     .target_static_save
    //     .iter()
    //     .any(|suffix| path_lower.ends_with(suffix))
    //     {
    //         // 判断不在目标资源存储列表内 则 流式响应
    //         return website_stream(Path(target_real_url), Extension(my_func)).await;
    //     }
    // }
    // 处理目标资源
    let target_path = format!("{target_domain}{uri}");
    let target_path_ = target_path.trim_end_matches("/");
    let target_path = MyFunc::path_clean(target_path_);
    let target_bucket;
    if target_lang == webconfig.info.to_lang {
        target_bucket = format!("target-{target_lang}");
    } else {
        target_bucket = format!("target-{}2{}", target_lang, webconfig.info.to_lang);
    }

    match client.stat_object(&target_bucket, &target_path).await {
        Ok(Some(stat)) => {
            // 存在目标资源
            // let content_type,t_url = stat.content_type().split("\n");
            let content_type;
            let t_url;
            if let Some((ct, url)) = stat.content_type().split_once("|") {
                content_type = ct.to_string();
                t_url = url.to_string();
                // println!("Content-Type: {}", content_type);
                // println!("URL: {}", t_url);
            } else {
                content_type = stat.content_type().to_string(); // 使用完整的 content_type
                t_url = "".to_string(); // 或者设置为默认 URL
            };
            // 生成缓存并返回
            return create_cache(
                &domain_info,
                &config_dict,
                state,
                &my_func,
                &client,
                &pgsql,
                table_name,
                uri,
                come_from_url,
                &target_bucket,
                &target_domain,
                &target_path,
                &cache_path,
                &content_type,
                &webconfig,
                0,
                &linecache,
                req,
            )
            .await;
        }
        Ok(None) | Err(_) => {
            // 不存在目标资源
            println!("{} 没有目标文件 开始爬取目标站", target_path);
            // 记录开始时间
            let download_start_time = Instant::now();

            // 自定义元数据
            match my_func.fetch_url(&target_real_url).await {
                Ok((content_type, file_bytes)) => {
                    // 记录结束时间
                    let download_end_time = Instant::now();
                    // 计算耗时（以毫秒为单位）
                    let download_duration_ms =
                        (download_end_time - download_start_time).as_millis();
                    // 将 HeaderValue 转换为字符串
                    let content_type_str = content_type.to_str().map_err(|_| {
                        println!("无法解析 content_type");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
                    // 处理html
                    let clean_text_bytes = if content_type_str.contains("html") {
                        let mut text: String;
                        // let tdk: String;
                        let trans_mode: bool;
                        if target_lang != webconfig.info.to_lang {
                            // 翻译一下
                            text = my_func.detect_encoding_and_decode(&file_bytes, &content_type);
                            let api_key = "csk-6mk2x5ke5x5ew9t839v4jwkk24dyryvxhyrjvhfjtefht8e2";
                            let model = "llama-4-scout-17b-16e-instruct";

                            let ai_trans = AiTrans::new(api_key, model, my_func.clone());
                            text = match ai_trans
                                .yd_translate_html(&text, &target_lang, &webconfig.info.to_lang)
                                .await
                            {
                                Ok(transed_text) => {
                                    println!("翻译成功");
                                    trans_mode = true;
                                    transed_text
                                }
                                Err(e) => {
                                    // 记录错误日志
                                    eprintln!("Translation failed: {}", e);
                                    // 可以选择重试或返回自定义错误
                                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                }
                            };
                            // let use_ip = my_func.get_random_element(&my_func.ips);
                            // match my_func
                            //     .yd
                            //     .web_trans(
                            //         &target_real_url,
                            //         &target_lang,
                            //         &webconfig.info.to_lang,
                            //         &use_ip,
                            //     )
                            //     .await
                            // {
                            //     Ok((transed_text, tdk)) => {
                            //         if !transed_text.ends_with("</html>") {
                            //             // 记录错误日志
                            //             eprintln!("翻译失败，返回了非html内容");
                            //             // 可以选择重试或返回自定义错误
                            //             return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            //         }
                            //         trans_mode = true;
                            //         text = transed_text;
                            //         if tdk.len() > 2 {
                            //             // 翻译TDK
                            //             match my_func
                            //                 .yd
                            //                 .trans(
                            //                     &tdk,
                            //                     &target_lang,
                            //                     &webconfig.info.to_lang,
                            //                     &use_ip,
                            //                 )
                            //                 .await
                            //             {
                            //                 Ok(transed_tdk_json) => {
                            //                     // text = transed_text;
                            //                     // println!("翻译测试：{:?}", transed_tdk_json);
                            //                     // 将 HashMap 的键值对提取到 Vec 中
                            //                     let mut entries: Vec<(String, String)> =
                            //                         transed_tdk_json.into_iter().collect();

                            //                     // 按键的长度从长到短排序
                            //                     entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

                            //                     // 打印排序后的键值对
                            //                     for (k, v) in entries {
                            //                         // println!("键: {}, 值: {}", k, v);
                            //                         text = text.replace(&k, &v);
                            //                     }
                            //                     // 全局替换tdk
                            //                     // for (k,v) in transed_tdk_json{
                            //                     //     text = text.replace(&k,&v);
                            //                     // }
                            //                     // text = transed_text;
                            //                     // trans_mode = true;
                            //                 }
                            //                 Err(e) => {
                            //                     // return Err(e);
                            //                     // 记录错误日志
                            //                     println!("Trans failed: {}", e);
                            //                     // 可以选择重试或返回自定义错误
                            //                     // return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            //                 }
                            //             }
                            //         }
                            //     }
                            //     Err(e) => {
                            //         // return Err(e);
                            //         // 记录错误日志
                            //         eprintln!("Translation failed: {}", e);
                            //         // 可以选择重试或返回自定义错误
                            //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            //     }
                            // }
                        } else {
                            // 不需要翻译 自动检测编码并转换为 UTF-8 字符串
                            text = my_func.detect_encoding_and_decode(&file_bytes, &content_type);
                            trans_mode = false;
                        }

                        println!("原html {}", text.len());

                        // 记录开始时间
                        let clean_start_time = Instant::now();
                        let cleaned_text = my_func.clean_html(&text, &target_domain, trans_mode);

                        // 记录结束时间
                        let clean_end_time = Instant::now();

                        // 计算耗时（以毫秒为单位）
                        let clean_duration_ms = (clean_end_time - clean_start_time).as_millis();
                        println!(
                            "新html {} 耗时：{} ms",
                            cleaned_text.len(),
                            clean_duration_ms
                        );

                        cleaned_text.into_bytes()
                    } else {
                        file_bytes
                    };

                    // 上传文件到 MinIO target目标
                    let target_path_key = KeyArgs::new(&target_path)
                        .content_type(Some(format!("{}|{}", content_type_str, target_real_url)));
                    match client
                        .put_object(&target_bucket, target_path_key, clean_text_bytes.into())
                        .await
                    {
                        Ok(_) => {
                            println!("{} 目标文件生成成功", target_path);
                            // 生成缓存并返回
                            return create_cache(
                                &domain_info,
                                &config_dict,
                                state,
                                &my_func,
                                &client,
                                &pgsql,
                                table_name,
                                uri,
                                come_from_url,
                                &target_bucket,
                                &target_domain,
                                &target_path,
                                &cache_path,
                                content_type_str,
                                &webconfig,
                                download_duration_ms,
                                &linecache,
                                req,
                            )
                            .await;
                        }
                        Err(e) => {
                            println!("{} 目标文件生成失败: {}", target_path, e);
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    }
                }
                Err(e) => {
                    // 访问目标站失败 则写入空文件
                    println!(
                        "访问目标站{} err:{} 写入空文件",
                        target_real_url,
                        e.as_u16()
                    );
                    if e.as_u16() != 429 {
                        // 上传文件到 MinIO
                        let target_path_key = KeyArgs::new(&target_path)
                            .content_type(Some(format!("{}|{}", e.as_u16(), target_real_url)));
                        match client
                            .put_object(&target_bucket, target_path_key, "".into())
                            .await
                        {
                            Ok(_) => {
                                println!("【{}】{} 目标空文件生成成功", target_bucket, target_path);
                                // 生成 SEO404 页面
                                if config_dict.seo_functions.seo_404_page {
                                    // 异步读取 HTML 文件
                                    let html_content = match read_to_string(std::path::Path::new(
                                        "templates/seo404.html",
                                    ))
                                    .await
                                    {
                                        Ok(content) => content,
                                        Err(e) => {
                                            eprintln!("Failed to read tag.html: {}", e);
                                            "".to_string()
                                        }
                                    };
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
                                    return Ok(Response::builder()
                                        .status(200)
                                        .header("Content-Type", "text/html; charset=utf-8")
                                        .body(Body::from(new_html))
                                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                                        .unwrap());
                                }
                            }
                            Err(e) => {
                                println!(
                                    "【{}】{} 目标空文件生成失败: {}",
                                    target_bucket, target_path, e
                                );
                            }
                        }
                    }
                    return Err(e);
                }
            }
        }
    }
}

async fn create_cache(
    domain_info: &HashMap<String, String>,
    config_dict: &Config,
    state: AppState,
    my_func: &MyFunc,
    client: &Arc<Minio>,
    pgsql: &Arc<PgsqlService>,
    table_name: String,
    uri: String,
    come_from_url: String,
    target_bucket: &str,
    target_domain: &str,
    target_path: &str,
    cache_path: &str,
    content_type: &str,
    webconfig: &WebsiteConf,
    download_time: u128,
    linecache: &Arc<AsyncLineCache>,
    req: Request,
) -> Result<Response, StatusCode> {
    let req_state = req.extensions().get::<RequestState>().unwrap();
    let is_index = uri == "/";
    // let table_name = domain_info["full_domain"].replace(".", "_"); // 表名
    let target_info = format!("{}://{}", target_bucket, target_path);
    // target_re 获取
    let mut target_re = TargetReplaceRules {
        all: ["待替换字符串 -> {关键词}".to_string()].to_vec(),
        index: ["待替换字符串 -> {关键词}".to_string()].to_vec(),
        page: ["待替换字符串 -> {关键词}".to_string()].to_vec(),
    };
    let target_re_path = format!("{}.toml", target_domain);
    match client.get_object("replace", &target_re_path).await {
        Ok(object) => {
            let content = object.text().await.unwrap();
            // 解析 TOML
            let parsed_config: Result<TargetReplaceRules, toml::de::Error> =
                toml::from_str(&content);
            match parsed_config {
                Ok(target_replace_rules) => {
                    // println!("target_replace_rules: {:?}", target_replace_rules);
                    target_re = target_replace_rules;
                }
                Err(e) => {
                    println!("Error parsing TOML: {}", e);
                }
            }
        }
        Err(_) => {
            // println!("{} 没有target_re配置文件", target_re_path);
        }
    }

    match client.get_object(target_bucket, target_path).await {
        Ok(object) => {
            // 判断是html则处理文本替换词后，上传到cache缓存
            if content_type.contains("html") {
                let content = object.text().await.unwrap();
                // 记录开始时间
                let start_time = Instant::now();

                let (replaced_text, internal_links) = my_func
                    .replace_html(
                        &content,
                        is_index,
                        &target_re,
                        webconfig,
                        config_dict,
                        req_state,
                        linecache, // req_state.domain_info["full_domain"].clone(),
                                   // req_state.domain_info["root_domain"].clone(),
                    )
                    .await;

                let mut new_html = replaced_text.clone();

                // 缓存 处理外链
                let mut result = String::with_capacity(new_html.len());
                let mut segments = new_html.split("【new_link】");
                if let Some(first) = segments.next() {
                    result.push_str(first);
                }
                for segment in segments {
                    let random_index =
                        rand::rng().random_range(0..config_dict.seo_functions.external_links.len());
                    let new_link = &config_dict.seo_functions.external_links[random_index];
                    result.push_str(new_link);
                    result.push_str(segment);
                }
                new_html = result;

                // 处理友链
                if config_dict.seo_functions.friend_link_count > 0
                    && !config_dict.seo_functions.friend_links.is_empty()
                {
                    let mut links_html = String::new();
                    for _ in 0..config_dict.seo_functions.friend_link_count {
                        let random_index = rand::rng()
                            .random_range(0..config_dict.seo_functions.friend_links.len());
                        // 返回随机元素
                        let random_s =
                            &format!("{}\n", config_dict.seo_functions.friend_links[random_index]);
                        // 生成时间戳 + 随机数（例如：1691234567_42）
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let random_num = rand::rng().random_range(1..1000);
                        let replacement = format!("#{}{}}}", timestamp, random_num);
                        let result = TIMESTAMP_REGEX.replace_all(random_s, replacement.as_str());

                        links_html.push_str(&result)
                    }
                    links_html.push_str("</body>");
                    // 使用正则表达式替换
                    new_html = BODY_FOOTER_REGEX
                        .replace(&new_html, &links_html)
                        .to_string();
                }
                // 标签处理
                let new_html = my_func
                    .tag_parse(
                        config_dict,
                        webconfig,
                        req_state,
                        pgsql,
                        linecache,
                        new_html,
                    )
                    .await;

                // 记录结束时间
                let end_time = Instant::now();

                // 计算耗时（以毫秒为单位）
                let duration_ms = (end_time - start_time).as_millis();
                println!("cache文本替换 耗时：{} ms", duration_ms);

                // println!("cache_path：{} 发现内链", cache_path);
                let table_name_ = table_name.clone();
                let cache_path_ = cache_path.to_string();
                let domain = domain_info["full_domain"].clone();
                let root_domain = domain_info["root_domain"].clone();
                let pgsql_ = pgsql.clone();
                let async_client = client.clone();
                let link_mapping = webconfig.info.link_mapping;

                let internal_links__ = internal_links.clone();
                let domain__ = domain_info["full_domain"].clone();
                let async_client__ = client.clone();
                let target_bucket__ = target_bucket.to_string();
                let target_domain__ = target_domain.to_string();

                if link_mapping {
                    // 启动异步任务 生成映射链接
                    spawn(async move {
                        // 记录开始时间
                        let start_time = Instant::now();
                        let mut count: usize = 0;
                        let mut success_count: usize = 0;
                        for (link_path, mapping_path) in internal_links {
                            count += 1;
                            // println!("link_path:{}\nmapping_path:{}", link_path, mapping_path);
                            if link_path.len() > 1 {
                                // 生成映射空文件
                                if mapping_path.len() > 0 {
                                    let internal_mapping_url = format!(
                                        "{}/{}",
                                        domain,
                                        mapping_path.trim_start_matches("/")
                                    );

                                    let minio_mapping_path =
                                        MyFunc::path_clean(&internal_mapping_url);

                                    // let object_exists = match check_object_exists(
                                    //     async_client.clone(),
                                    //     "cache",
                                    //     &minio_mapping_path,
                                    // )
                                    // .await
                                    // {
                                    //     Some(true) => true,
                                    //     Some(false) => false,
                                    //     None => false,
                                    // };
                                    // 检查 URL 是否已经在缓存中
                                    let object_exists = match pgsql_
                                        .cache_path_exists(&table_name_, &minio_mapping_path)
                                        .await
                                    {
                                        Ok(true) => true,
                                        Ok(false) => false,
                                        Err(_) => false,
                                    };

                                    if !object_exists {
                                        let come_from_url_copy = format!(
                                            "http://{}/{}",
                                            domain,
                                            mapping_path.trim_start_matches("/")
                                        );

                                        let mapping_url =
                                            MyFunc::encode_url_path(&come_from_url_copy);
                                        let mapping_uri = format!(
                                            "/{}",
                                            MyFunc::encode_url_path(&link_path)
                                                .trim_start_matches('/')
                                        );
                                        let mut data = HashMap::new(); // 要插入的数据
                                        data.insert("cache_path", minio_mapping_path.as_str());
                                        data.insert("url", mapping_url.as_str());
                                        data.insert("uri", mapping_uri.as_str());
                                        data.insert("target", "");
                                        data.insert("title", "");
                                        data.insert("keywords", "");
                                        data.insert("description", "");
                                        data.insert("domain", domain.as_str());
                                        data.insert("root_domain", root_domain.as_str());
                                        data.insert("page_type", "映射");
                                        data.insert("source", "");

                                        // pgsql 写入 映射数据
                                        match pgsql_
                                            .insert_or_create_website_cache(
                                                &table_name_,
                                                data,
                                                false,
                                            )
                                            .await
                                        {
                                            Ok(()) => {
                                                // println!("映射数据{} 插入成功", minio_mapping_path);
                                                success_count += 1;
                                            }
                                            Err(status) => {
                                                println!("映射数据{} 插入失败", minio_mapping_path);
                                            }
                                        }
                                        // // 不存在映射文件则上传
                                        // match async_client
                                        //     .put_object("cache", cache_path_key, "".into())
                                        //     .await
                                        // {
                                        //     Ok(_) => {
                                        //         // println!(
                                        //         //     "【cache】{} 映射文件生成成功",
                                        //         //     minio_mapping_path
                                        //         // );
                                        //         success_count += 1;
                                        //     }
                                        //     Err(e) => {
                                        //         println!(
                                        //             "【cache】{} 映射文件生成失败: {}",
                                        //             minio_mapping_path, e
                                        //         );
                                        //     }
                                        // }
                                    };
                                }
                            }
                        }
                        // 记录结束时间
                        let end_time = Instant::now();

                        // 计算耗时（以毫秒为单位）
                        let duration_ms = (end_time - start_time).as_millis();
                        println!(
                            "{}组链接 生成映射空文件{}个 耗时：{} ms",
                            count, success_count, duration_ms
                        );
                        // 这里可以执行一些后台操作，例如发送邮件、写入数据库等
                    });
                }

                let ua = req
                    .headers()
                    .get(USER_AGENT)
                    .and_then(|ua| ua.to_str().ok())
                    .unwrap_or("User-Agent not found");

                // if ua != "myself"{
                //     // 启动异步任务 预下载文件
                //     spawn(async move {
                //         // 记录开始时间
                //         let start_time = Instant::now();
                //         for (link_path, mapping_path) in internal_links__ {
                //             // println!("link_path:{}\nmapping_path:{}", link_path, mapping_path);
                //             if link_path.len() > 1 {
                //                 let target_internal_url = format!(
                //                     "{}/{}",
                //                     target_domain__,
                //                     link_path.trim_start_matches("/")
                //                 );
                //                 let target_minio_path = MyFunc::path_clean(
                //                     &MyFunc::encode_url_path(&target_internal_url),
                //                 );
                //                 let target_object_exists = match check_object_exists(
                //                     async_client__.clone(),
                //                     &target_bucket__,
                //                     &target_minio_path,
                //                 )
                //                 .await
                //                 {
                //                     Some(true) => true,
                //                     Some(false) => false,
                //                     None => false,
                //                 };

                //                 if !target_object_exists {
                //                     // 不存在目标资源 判断是否存在缓存文件
                //                     let internal_url = format!(
                //                         "{}/{}",
                //                         domain__,
                //                         link_path.trim_start_matches("/")
                //                     );
                //                     let minio_path =
                //                         MyFunc::path_clean(&MyFunc::encode_url_path(&internal_url));
                //                     let object_exists = match check_object_exists(
                //                         async_client__.clone(),
                //                         "cache",
                //                         &minio_path,
                //                     )
                //                     .await
                //                     {
                //                         Some(true) => true,
                //                         Some(false) => false,
                //                         None => false,
                //                     };
                //                     if !object_exists {
                //                         // 都不存在的情况下 开始预下载
                //                         let url = if domain__ == "www.localhost.com" {
                //                             format!("https://fluffy-memory-xpw5769x7qp3qwg-16888.app.github.dev/{}", link_path.trim_start_matches("/"))
                //                         } else {
                //                             format!("http://{}", internal_url)
                //                         };

                //                         // 检查 URL 是否已经在缓存中
                //                         if state.fetching_urls.get(&url).is_some() {
                //                             println!("{} 已在缓存中，跳过预下载", url);
                //                         } else {
                //                             println!("访问：{} 开始预下载", url);
                //                             // // 遍历缓存中的所有数据
                //                             // for (key, value) in state.fetching_urls.iter() {
                //                             //     println!("Key: {}, Value: {:?}", key, value);
                //                             // }
                //                             // 将 URL 添加到缓存中，表示正在处理
                //                             state.fetching_urls.insert(url.clone(), ());
                //                             spawn(async move {
                //                                 match MyFunc::fetch_my_url(&url).await {
                //                                     Ok((_content_type, bytes)) => {
                //                                         println!(
                //                                             "{} 预下载成功 {}",
                //                                             url,
                //                                             bytes.len()
                //                                         )
                //                                     }
                //                                     Err(e) => {
                //                                         println!("{} 预下载失败 {}", url, e)
                //                                     }
                //                                 };
                //                             });
                //                         }
                //                     }
                //                 }
                //                 // else{
                //                 //     println!(
                //                 //         "[{}]{} => {} 目标文件存在\n跳过访问：{}",
                //                 //         target_bucket__, internal_url, minio_path, url
                //                 //     );
                //                 // }
                //             }
                //         }
                //         // 记录结束时间
                //         let end_time = Instant::now();

                //         // 计算耗时（以毫秒为单位）
                //         let duration_ms = (end_time - start_time).as_millis();
                //         println!("异步预下载页面 耗时：{} ms", duration_ms);
                //         // 这里可以执行一些后台操作，例如发送邮件、写入数据库等
                //     });
                // }

                // let file_content = new_html.clone().into_bytes();
                // let file_content = replaced_text.clone().into_bytes();

                // 上传至cache
                // let cache_path = MyFunc::encode_url_path(&come_from_url);
                // let cache_path_key = KeyArgs::new(cache_path)
                //     .content_type(Some(cache_path));

                // cache_path TEXT UNIQUE,        // 唯一URL
                // url TEXT,                      // 唯一URL
                // uri TEXT,                      // 唯一URL
                // target TEXT,                   // 目标内容
                // title TEXT,                    // 页面标题
                // keywords TEXT,                 // 关键词
                // description TEXT,              // 描述
                // domain VARCHAR(255),           // 域名
                // root_domain VARCHAR(255),      // 根域名
                // created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  // 创建时间
                // updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  // 更新时间
                // page_type VARCHAR(50),         // 页面类型 缓存/映射/目录/静态
                // source TEXT                    // 来源
                // 添加数据（字段需与表结构匹配）

                let metadata = MyFunc::extract_metadata(new_html.as_str());
                // println!("metadata: {:?}", metadata);

                // pgsql 写入缓存
                let mut data = HashMap::new(); // 要插入的数据
                data.insert("cache_path", cache_path);
                data.insert("url", come_from_url.as_str());
                data.insert("uri", uri.as_str());
                data.insert("target", target_info.as_str());
                data.insert("title", metadata.title.as_deref().unwrap_or(""));
                data.insert("keywords", metadata.keywords.as_deref().unwrap_or(""));
                data.insert("description", metadata.description.as_deref().unwrap_or(""));
                data.insert("domain", domain_info["full_domain"].as_str());
                data.insert("root_domain", domain_info["root_domain"].as_str());
                data.insert("page_type", "缓存");
                println!("data: {:?}", data);
                data.insert("source", new_html.as_str());

                match pgsql
                    .insert_or_create_website_cache(&table_name, data, false)
                    .await
                {
                    Ok(()) => {
                        println!("缓存数据{} 插入成功", cache_path);
                        let download_time_str = format!("{} ms", download_time);
                        return Ok(Response::builder()
                            .header("Content-Type", "text/html; charset=utf-8")
                            .header("M-Page-Mode", "new-html")
                            .header("M-download-time", download_time_str)
                            .body(Body::from(new_html))
                            .unwrap());
                    }
                    Err(status) => {
                        println!("缓存数据{} 插入失败", cache_path);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            } else if content_type.chars().all(|c| c.is_digit(10)) {
                // println!("错误页面{}", content_type);
                // 将 u16 转换为 StatusCode
                match StatusCode::from_u16(u16::from_str(content_type).unwrap_or(500)) {
                    Ok(code) => {
                        if config_dict.seo_functions.seo_404_page {
                            // 异步读取 HTML 文件
                            let html_content =
                                match read_to_string(std::path::Path::new("templates/seo404.html"))
                                    .await
                                {
                                    Ok(content) => content,
                                    Err(e) => {
                                        eprintln!("Failed to read tag.html: {}", e);
                                        "".to_string()
                                    }
                                };
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
                            return Ok(Response::builder()
                                .status(200)
                                .header("Content-Type", "text/html; charset=utf-8")
                                .body(Body::from(new_html))
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                                .unwrap());

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
                        } else {
                            return Err(code);
                        }
                    } // 返回解析后的状态码
                    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR), // 如果无效，返回 500
                }
            } else {
                // 判断不是html则 直接上传到cache 在content_type中写上目标静态链接
                // let cache_link_to_target_path = format!("{}://{}", target_bucket, target_path);
                // println!(
                //     "【cache】 {} 写入空文件 指定content_type为：{}",
                //     cache_path, target_info
                // );
                // // 上传文件到 MinIO
                // let cache_path_key =
                //     KeyArgs::new(cache_path).content_type(Some(target_info));
                // match client.put_object("cache", cache_path_key, "".into()).await {
                //     Ok(_) => {
                //         println!("【cache】{} 缓存映射文件生成成功", cache_path);
                //         return Ok(Response::builder()
                //             .header("Content-Type", content_type)
                //             .header("M-Page-Mode", "new-static")
                //             .body(Body::from_stream(object.bytes_stream()))
                //             .unwrap());
                //     }
                //     Err(e) => {
                //         println!("【cache】{} 缓存映射文件生成失败: {}", cache_path, e);
                //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
                //     }
                // }

                // pgsql 写入空缓存 指向目标静态资源
                let mut data = HashMap::new(); // 要插入的数据
                data.insert("cache_path", cache_path);
                data.insert("url", come_from_url.as_str());
                data.insert("uri", uri.as_str());
                data.insert("target", target_info.as_str());
                data.insert("title", "");
                data.insert("keywords", "");
                data.insert("description", "");
                data.insert("domain", domain_info["full_domain"].as_str());
                data.insert("root_domain", domain_info["root_domain"].as_str());
                data.insert("page_type", "静态");
                data.insert("source", "");
                match pgsql
                    .insert_or_create_website_cache(&table_name, data, false)
                    .await
                {
                    Ok(()) => {
                        println!("缓存数据 静态目标{} 插入成功", cache_path);
                    }
                    Err(status) => {
                        println!("缓存数据 静态目标{} 插入失败", cache_path);
                        // return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
                return Ok(Response::builder()
                    .header("Content-Type", content_type)
                    .header("M-Page-Mode", "new-static")
                    .body(Body::from_stream(object.bytes_stream()))
                    .unwrap());
            }
        }
        Err(_) => {
            println!("不存在目标资源: {}", target_path);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

// /// 单独的函数：获取或生成配置文件
// pub async fn fetch_or_create_config(
//     is_www: bool,
//     config_dict: &Config,
//     my_func: &Arc<MyFunc>,
//     pgsql: &Arc<PgsqlService>,
//     config_path: &str,
//     domain: &str,
// ) -> Result<WebsiteConf, StatusCode> {
//     let mut conditions: HashMap<&str, &str> = HashMap::new();
//     conditions.insert("domain", domain);

//     match pgsql
//         .fetch_data(
//             "website_config",
//             &[],
//             conditions,
//             None,
//             Some(1),
//             Some(1),
//             None,
//             None,
//         )
//         .await
//     {
//         Ok((rows, total)) => {
//             // 将 PgRow 转换为可序列化的格式
//             let items: Vec<_> = rows
//     .into_iter()
//     .map(|row| {
//         json!({
//             "id": row.get::<i32, _>("id"),
//             "website_info": {
//                 "domain": row.get::<Option<String>, _>("domain").unwrap_or_default(),
//                 "subdomain": row.get::<Option<String>, _>("subdomain").unwrap_or_default(),
//                 "root_domain": row.get::<Option<String>, _>("root_domain").unwrap_or_default(),
//                 "target": row.get::<Option<String>, _>("target").unwrap_or_default(),
//                 "to_lang": row.get::<Option<String>, _>("to_lang").unwrap_or_default(),
//                 "title": row.get::<Option<String>, _>("title").unwrap_or_default(),
//                 "keywords": row.get::<Option<String>, _>("keywords").unwrap_or_default(),
//                 "description": row.get::<Option<String>, _>("description").unwrap_or_default(),
//                 "link_mapping": row.get::<bool, _>("link_mapping"),
//             },
//             "replace_rules": {
//                 "replace_mode": row.get::<i32, _>("replace_mode"),
//                 "replace_rules_all": row.get::<Option<Vec<String>>, _>("replace_rules_all").unwrap_or_default(),
//                 "replace_rules_index": row.get::<Option<Vec<String>>, _>("replace_rules_index").unwrap_or_default(),
//                 "replace_rules_page": row.get::<Option<Vec<String>>, _>("replace_rules_page").unwrap_or_default(),
//             },
//             "mulu_config": {
//                 "mulu_tem_max": row.get::<i32, _>("mulu_tem_max"),
//                 "mulu_mode": row.get::<Option<String>, _>("mulu_mode").unwrap_or_default(),
//                 "mulu_static": row.get::<bool, _>("mulu_static"),
//                 "mulu_template": row.get::<Option<Vec<String>>, _>("mulu_template").unwrap_or_default(),
//                 "mulu_custom_header": row.get::<Option<Vec<String>>, _>("mulu_custom_header").unwrap_or_default(),
//                 "mulu_keywords_file": row.get::<Option<Vec<String>>, _>("mulu_keywords_file").unwrap_or_default(),
//             },
//             "include_info": {
//                 "google_include_info": row.get::<Option<Vec<String>>, _>("google_include_info").unwrap_or_default(),
//                 "bing_include_info": row.get::<Option<Vec<String>>, _>("bing_include_info").unwrap_or_default(),
//                 "baidu_include_info": row.get::<Option<Vec<String>>, _>("baidu_include_info").unwrap_or_default(),
//                 "sogou_include_info": row.get::<Option<Vec<String>>, _>("sogou_include_info").unwrap_or_default(),
//             },
//             "homepage_update_time": row.get::<i32, _>("homepage_update_time"),
//             "created_at": row.get::<DateTime<Utc>, _>("created_at"),
//             "updated_at": row.get::<DateTime<Utc>, _>("updated_at"),
//         })
//     })
//     .collect();
//             let website_config: WebsiteConf = match items.into_iter().next() {
//                 Some(item) => serde_json::from_value(item).map_err(|e| {
//                     eprintln!("Failed to deserialize: {}", e);
//                     StatusCode::INTERNAL_SERVER_ERROR
//                 })?,
//                 None => {
//                     // eprintln!("No items found");
//                     println!("{} 没有配置文件", domain);
//                     // ----------------------------------------------------------
//                     if is_www {
//                         if !config_dict.website_settings.auto_site_building {
//                             println!("{} 自动建站已经关闭", domain);
//                             return Err(StatusCode::BAD_REQUEST);
//                         }
//                     } else {
//                         if config_dict.website_settings.auto_site_building
//                             && !config_dict.website_settings.pan_site_auto_site_building
//                         {
//                             println!("{} 泛站自动建站已经关闭", domain);
//                             return Err(StatusCode::BAD_REQUEST);
//                         }
//                     }
//                     println!("{} 自动生成配置文件", config_path);
//                     // 检测域名归属
//                     let name = config_dict.program_info.program_name.clone();
//                     let check_url = format!("http://{}/_api/program_name", domain);
//                     match my_func.fetch_url(&check_url).await {
//                         Ok((content_type, file_bytes)) => {
//                             let text =
//                                 my_func.detect_encoding_and_decode(&file_bytes, &content_type);
//                             println!("访问结果:{}\n自己获取:{}", text, name);
//                             if text == name {
//                                 println!("{} 域名归属正确", domain);
//                             } else {
//                                 println!("{} 域名归属错误", domain);
//                                 return Err(StatusCode::INTERNAL_SERVER_ERROR);
//                             }
//                         }
//                         Err(e) => {
//                             println!("{} 域名归属错误", domain);
//                             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//                         }
//                     }
//                     return Err(StatusCode::NOT_FOUND);
//                 }
//             };

//             Ok(website_config)
//         } // 表存在，直接返回数据
//         Err(status) => {
//             // 报错
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     }
// }
