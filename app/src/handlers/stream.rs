use std::{net::IpAddr, sync::Arc};

use crate::functions::func::MyFunc;

use axum::{
    body::Body,
    extract::{Path, Query, Request},
    http::{header::CONTENT_TYPE, StatusCode},
    response::Response,
    Extension,
};

use bytes::Bytes;
use futures::StreamExt;
use minio_rsc::Minio;
use rand_user_agent::UserAgent;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
// use tokio_util::io::StreamReader;
use tracing::{error, info};

// pub async fn minio_stream(Path(url): Path<String>, client: Minio) -> Result<Response, StatusCode> {
//     info!("path: {url}");
//     // url
//     let bucket_name = "config";
//     let object_name = "www.baidu.com.yml";

//     match client.get_object(bucket_name, object_name).await {
//         Ok(object) => {
//             // 将 MinIO 对象的字节流转换为 tokio::io::AsyncRead 流
//             let stream = object.bytes_stream();
//             let body = Body::from_stream(stream);
//             Ok(Response::builder()
//                 .header("Content-Type", "application/octet-stream")
//                 .body(body)
//                 .unwrap())
//         }
//         Err(_) => Err(StatusCode::NOT_FOUND),
//     }
// }

// #[derive(Deserialize)]
// pub struct Params {
//     dir: Option<String>,
// }

// pub async fn minio_stream(
//     // Query(params): Query<Params>,
//     Path(url): Path<String>,
//     // Extension(client): Extension<Arc<Minio>>,
//     Extension(minio_client): Extension<Arc<MinioClient>>,
//     req: Request,
// ) -> Result<Response, StatusCode> {
//     // info!("minio path: {url}");
//     // 获取完整的 URI
//     let uri = req.uri().to_string();
//     // let (bucket_name,object_name) = uri.split_once("/");
//     let (bucket_name, object_name) = uri.trim_matches('/').split_once('/').unwrap_or((&uri, ""));
//     // let bucket_name = params.dir.unwrap_or("cache".to_string());
//     // let object_name = url.trim_start_matches('/');

//     // 获取对象的元数据
//     match minio_client.get_object(bucket_name, object_name).await {
//         Ok(object) => {
//             let content_type = object.metadata.content_type;
//             let body = Body::from(object.data);
//             Ok(Response::builder()
//                 .header("Content-Type", content_type)
//                 .body(body)
//                 .unwrap())
//         }
//         Err(_) => Err(StatusCode::NOT_FOUND),
//     }
// }

pub async fn website_stream(
    Path(url): Path<String>,
    Extension(my_func): Extension<Arc<MyFunc>>,
) -> Result<Response, StatusCode> {
    // 规范化URL格式
    let target_url = if url.starts_with("http") {
        url
    } else {
        format!("http://{}", url)
    };

    // 获取用户代理字符串和随机IP
    let rua = UserAgent::pc().to_string();
    let use_ip = my_func.get_random_element(&my_func.ips);

    // 解析IP地址，添加更好的错误处理
    let ip: IpAddr = match use_ip.parse() {
        Ok(ip) => ip,
        Err(_) => {
            error!("IP地址解析失败: {}", use_ip);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 创建HTTP客户端，添加超时配置
    let client = match Client::builder()
        .danger_accept_invalid_certs(true) // 禁用证书验证（注意：生产环境需谨慎使用）
        .local_address(ip.clone())
        .timeout(std::time::Duration::from_secs(30)) // 添加30秒超时
        .gzip(true) // 启用 gzip 解压缩
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("创建HTTP客户端失败: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 发送请求并处理可能的错误
    let response = match client
        .get(&target_url)
        .header("user-agent", rua.clone())
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("【{}】目标访问 {} {}",ip, rua, e);
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 获取Content-Type，设置默认值
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // 创建有缓冲的通道
    let (tx, rx) = mpsc::channel::<Result<Bytes, reqwest::Error>>(100);

    // 异步处理流式响应
    tokio::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    if let Err(e) = tx.send(Ok(bytes)).await {
                        error!("发送字节流失败: {}", e);
                        break;
                    }
                }
                Err(err) => {
                    if let Err(e) = tx.send(Err(err)).await {
                        error!("发送错误失败: {}", e);
                    }
                    break;
                }
            }
        }
        // tx会在离开作用域时自动drop，无需显式关闭
    });

    // 创建流式响应体
    let body_stream = ReceiverStream::new(rx);
    let stream_body = Body::from_stream(body_stream);

    // 构建响应
    match Response::builder()
        .header("Content-Type", content_type)
        .header("M-Page-Mode", "stream")
        .header("Cache-Control", "public, max-age=604800") // 设置浏览器缓存
        .body(stream_body)
    {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("构建响应失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// pub async fn website_stream(
//     Path(url): Path<String>,
//     Extension(my_func): Extension<Arc<MyFunc>>,
// ) -> Result<Response, StatusCode> {
//     // info!("path: {url}");
//     let target_url = if url.starts_with("http") {
//         url
//     } else {
//         format!("http://{}", url)
//     };

//     let rua = UserAgent::pc().to_string();
//     let use_ip = my_func.get_random_element(&my_func.ips);
//     // 解析指定的出口 IP 地址
//     let ip: IpAddr = use_ip
//         .parse()
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     // info!("使用IP：{} 流式响应目标URL: {}", ip, target_url);
//     // 创建 HTTP 客户端，并指定出口 IP
//     let client = Client::builder()
//         .danger_accept_invalid_certs(true) // 禁用证书验证
//         .local_address(ip) // 设置出口 IP
//         .build()
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     let response = client
//         .get(&target_url)
//         .header("user-agent", rua)
//         .send()
//         .await
//         .unwrap();
//     let content_type = response
//         .headers()
//         .get(CONTENT_TYPE)
//         .and_then(|v| v.to_str().ok())
//         .unwrap_or("application/octet-stream")
//         .to_string(); // 克隆 Content-Type 值

//     let (tx, rx) = mpsc::channel::<Result<Bytes, reqwest::Error>>(100);
//     tokio::spawn(async move {
//         let mut stream = response.bytes_stream();
//         // let mut total_bytes = 0;
//         while let Some(item) = stream.next().await {
//             match item {
//                 Ok(bytes) => {
//                     // total_bytes += bytes.len();
//                     // info!("Downloaded {} bytes", total_bytes);/
//                     if tx.send(Ok(bytes)).await.is_err() {
//                         error!("Failed to send bytes through channel");
//                         break;
//                     }
//                 }
//                 Err(err) => {
//                     error!("Error while streaming bytes: {}", err);
//                     break;
//                 }
//             }
//         }
//     });

//     let body_stream = ReceiverStream::new(rx);
//     let stream_body = Body::from_stream(body_stream);

//     Ok(Response::builder()
//         .header("Content-Type", content_type)
//         .header("M-Page-Mode", "stream")
//         .header("Cache-Control", "public, max-age=604800") // 流式静态文件 客户端缓存设置
//         .body(stream_body)
//         .unwrap())
// }

pub async fn download_website(
    Path(url): Path<String>,
    Extension(my_func): Extension<Arc<MyFunc>>,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    // info!("path: {url}");

    let target_url = if url.starts_with("http") {
        url
    } else {
        format!("http://{}", url)
    };
    // info!("目标URL: {}", target_url);

    // 调用封装的网络请求函数
    let (content_type, bytes) = my_func.fetch_url(&target_url).await?;

    // 计算下载大小和耗时
    let size_in_mb = bytes.len() as f64 / (1024.0 * 1024.0);
    let duration = start.elapsed();
    // info!(
    //     "URL: {} {:.2}MB 已下载，耗时: {:?}",
    //     target_url, size_in_mb, duration
    // );

    // 构建响应
    Ok(Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(Body::from(bytes))
        .unwrap())
}
