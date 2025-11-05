use crate::domain_info_from_domain;
use crate::functions::func::MyFunc;
use crate::my_const::{
    AD_JS_CODE, BODY_FOOTER_REGEX, BODY_HEADER_REGEX, HEAD_FOOTER_REGEX, HEAD_HEADER_REGEX,
    SEARCH_URLS, SECRET, SPIDERS_DICT,
};
use crate::Claims;
use crate::Config;
use crate::PgsqlService;
use crate::RequestState;
use axum::{
    body::{self},
    extract::Request,
    http::{header::CONTENT_TYPE, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use linecache::AsyncLineCache;
// use regex::Regex;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::{f32::consts::E, net::IpAddr};
use tracing::info;
// use indexmap::IndexMap;

pub async fn middleware(
    Extension(config): Extension<Arc<RwLock<Config>>>,
    Extension(pgsql): Extension<Arc<PgsqlService>>,
    Extension(my_func): Extension<Arc<MyFunc>>,
    Extension(linecache): Extension<Arc<AsyncLineCache>>,
    mut req: Request,
    next: Next,
) -> Response {
    // 请求处理前逻辑：记录开始时间
    let start_time = Instant::now();

    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // 获取 客户端真实 IP
    let client_ip: String;

    // 获取 X-Forwarded-For 请求头
    let x_forwarded_for = headers
        .get("x-forwarded-for") // 直接写请求头名称
        .and_then(|h| h.to_str().ok());
    match x_forwarded_for {
        Some(ip_list) => {
            // 解析 IP 列表，取第一个 IP 作为客户端真实 IP
            client_ip = ip_list.split(',').next().unwrap_or("").trim().to_string();
        }
        None => {
            // 如果没有 X-Forwarded-For 头，使用远程地址
            // 构造并返回一个自定义的 Response
            let response = Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(format!("客户端IP获取失败").into())
                .unwrap();
            return response;
        }
    }

    // 处理 后台与静态路径
    let url_path = uri.path();
    if ["/_/", "/_api", "/_static/"]
        .iter()
        .any(|prefix| url_path.starts_with(prefix))
    {
        // 处理 _api_
        if url_path.starts_with("/_api_") && !url_path.starts_with("/_api_/logs") {
            // 获取 Authorization 头部
            let auth_header = match headers.get("Authorization") {
                Some(header) => header.to_str().unwrap_or(""),
                None => {
                    return Response::builder()
                        .status(401)
                        .body(format!("Token 丢失 from_ip: {}，请重新登录", client_ip).into())
                        .unwrap();
                }
            };

            // 移除 Bearer 前缀
            let token = auth_header.replace("Bearer ", "");

            // 配置 JWT 解码
            // let secret = "Mirror-Elf-R888888"; // 示例密钥，实际中应从环境变量等安全位置获取
            let key = jsonwebtoken::DecodingKey::from_secret(SECRET.as_bytes());
            let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
            match jsonwebtoken::decode::<Claims>(&token, &key, &validation) {
                Ok(_) => {
                    // Token 验证成功
                    // println!("Token 验证成功!");
                }
                Err(err) => match err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        return Response::builder()
                            .status(401)
                            .body(format!("Token 已过期，请重新登录").into())
                            .unwrap();
                    }
                    _ => {
                        return Response::builder()
                            .status(401)
                            .body(format!("无效的 Token，请重新登录").into())
                            .unwrap();
                    }
                },
            }
        }
        return next.run(req).await;
    }

    // 获取 config
    let config_dict = config.read().unwrap().clone();

    // 获取 req_referer
    let req_referer: String;

    // 获取 referer 请求头 判断来路跳广告
    let referer = headers
        .get("referer") // 直接写请求头名称
        .and_then(|h| h.to_str().ok());
    match referer {
        Some(referer_str) => {
            req_referer = referer_str.to_string();
            if config_dict.ad_policy.search_referrer_jump_ad {
                if SEARCH_URLS.iter().any(|u| req_referer.contains(u)) {
                    // 跳转到 广告url
                    return Redirect::permanent(&config_dict.ad_policy.ad_url).into_response();
                }
            }
        }
        None => {
            req_referer = "-".to_string();
        }
    }

    // 处理 IP黑名单
    if config_dict.access_policy.ip_banlist.contains(&client_ip) {
        let response = Response::builder()
            .status(403)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(format!("客户端IP异常：{}", client_ip).into())
            .unwrap();
        return response;
    }

    // 获取 客户端UA
    let client_ua: String;
    // 获取 UA头
    let user_agent = headers
        .get("user-agent") // 直接写请求头名称
        .and_then(|h| h.to_str().ok());
    match user_agent {
        Some(ua) => {
            // 解析 IP 列表，取第一个 IP 作为客户端真实 IP
            client_ua = ua.trim().to_string();
        }
        None => {
            // 如果没有 X-Forwarded-For 头，使用远程地址
            // 构造并返回一个自定义的 Response
            let response = Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(format!("客户端UA获取失败").into())
                .unwrap();
            return response;
        }
    }
    // 处理 UA黑名单
    let contains_any = config_dict
        .access_policy
        .ua_banlist
        .iter()
        .any(|ban_ua| client_ua.contains(ban_ua));
    if contains_any {
        let response = Response::builder()
            .status(403)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(format!("客户端UA异常：{}", client_ua).into())
            .unwrap();
        return response;
    }

    // 获取 https 或 http
    let scheme = uri.scheme_str().unwrap_or("http");

    // 获取 host
    let host_info = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // 解析 Host 以获取主机名和端口号
    let (host, _port) = if let Some(colon_pos) = host_info.find(':') {
        // 如果 Host 包含端口号
        let (host_part, port_part) = host_info.split_at(colon_pos);
        (host_part, Some(port_part.trim_start_matches(':')))
    } else {
        // 如果 Host 不包含端口号
        (host_info, None)
    };

    // 处理 强制绑定域名
    if config_dict.access_policy.forced_domain_binding {
        match linecache.get_lines("doc/bind_domain.txt").await {
            Ok(Some(bind_domains)) => {
                println!("bind_domains:{:?}", bind_domains);
                if !bind_domains
                    .iter()
                    .any(|domain| host.contains(domain.as_str()))
                {
                    // info!("{host} 非绑定域名 已拦截 【中间件】");
                    // 构造并返回一个自定义的 Response
                    let response = Response::builder()
                        .status(403)
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .body(format!("非绑定域名: {}", host).into())
                        .unwrap();
                    return response;
                }
            }
            Ok(None) => {
                println!("文件未找到或为空");
            }
            Err(e) => {
                println!("读取文件时发生错误：{}", e);
            }
        }
    }

    // 处理 IP来路
    let is_ip = host.parse::<IpAddr>().is_ok();
    if !config_dict.access_policy.ip_site_referrer {
        // 判断 host 是否为 IP 地址
        if is_ip {
            // info!("{host} 非法HOST【IP类型】 已拦截 【中间件】");
            // 构造并返回一个自定义的 Response
            let response = Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(format!("非法HOST【IP类型】: {}", host).into())
                .unwrap();
            return response;
        } else if !host.contains(".") {
            // info!("{host} 非法HOST 已拦截 【中间件】");
            // 构造并返回一个自定义的 Response
            let response = Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(format!("非法HOST: {}", host).into())
                .unwrap();
            return response;
        }
    }

    let domain = if host.eq_ignore_ascii_case("localhost") {
        "www.localhost.com".to_string()
    } else {
        host.to_ascii_lowercase()
    };
    let domain_info = domain_info_from_domain(&domain);

    if !is_ip {
        // 获取 域名信息
        // let domain_info = domain_info_from_domain(host);

        // 处理 根域名301跳转到www
        if domain_info["subdomain"] == "" && host.contains(".") {
            // 跳转到 www.
            let goto_url = format!("{}://www.{}{}", scheme, domain_info["root_domain"], uri);
            // 返回 301 永久重定向响应
            return Redirect::permanent(&goto_url).into_response();
        }

        // 处理 泛站来路
        if !config_dict.access_policy.pan_site_referrer {
            if domain_info["subdomain"] != "www" {
                // info!("{host} 非法HOST【泛站类型】 已拦截 【中间件】");
                // 构造并返回一个自定义的 Response
                let response = Response::builder()
                    .status(403)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(format!("非法HOST【泛站类型】: {}", host).into())
                    .unwrap();
                return response;
            }
        }

        // 处理 自动https
        // if scheme == "http" && domain_info["subdomain"] == "www"{
        //     if config_dict.website_settings.auto_https_certificate{
        //         // 返回 301 永久重定向响应
        //         let goto_url = format!("https://{}{}", host, req.uri());
        //         return Redirect::permanent(&goto_url).into_response();
        //     }
        // }
    }

    // 获取 蜘蛛名称
    let mut spider: Option<String> = None; // 使用 Option 初始化

    let mut spidername = "".to_string();
    for (spider_name_, keywords) in SPIDERS_DICT.iter() {
        if keywords.iter().any(|keyword| client_ua.contains(keyword)) {
            spidername = spider_name_.to_string();
            spider = Some(spider_name_.to_string()); // 赋值为 Some
            break;
        }
    }

    // 处理 蜘蛛策略
    let ua_fuck_response = Response::builder()
        .status(502)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(format!("").into())
        .unwrap();

    match spider {
        Some(spider_name) => match spider_name.as_str() {
            "百度蜘蛛" => {
                if !config_dict.spider_policy.baidu_spider {
                    return ua_fuck_response;
                }
            }
            "搜狗蜘蛛" => {
                if !config_dict.spider_policy.sogou_spider {
                    return ua_fuck_response;
                }
            }
            "神马蜘蛛" => {
                if !config_dict.spider_policy.yisou_spider {
                    return ua_fuck_response;
                }
            }
            "头条蜘蛛" => {
                if !config_dict.spider_policy.byte_spider {
                    return ua_fuck_response;
                }
            }
            "必应蜘蛛" => {
                if !config_dict.spider_policy.bing_spider {
                    return ua_fuck_response;
                }
            }
            "360蜘蛛" => {
                if !config_dict.spider_policy.so_spider {
                    return ua_fuck_response;
                }
            }
            "谷歌图片蜘蛛" => {
                if !config_dict.spider_policy.google_img_spider {
                    return ua_fuck_response;
                }
            }
            "谷歌蜘蛛" => {
                if !config_dict.spider_policy.google_spider {
                    return ua_fuck_response;
                }
            }
            "夸克蜘蛛" => {
                if !config_dict.spider_policy.quark_spider {
                    return ua_fuck_response;
                }
            }
            "雅虎蜘蛛" => {
                if !config_dict.spider_policy.yahoo_spider {
                    return ua_fuck_response;
                }
            }
            "其它蜘蛛" => {
                if !config_dict.spider_policy.other_spider {
                    return ua_fuck_response;
                }
            }
            _ => {
                if !config_dict.spider_policy.user {
                    return ua_fuck_response;
                }
                if config_dict.ad_policy.regular_ua_jump_ad {
                    // 跳转到 广告url
                    return Redirect::permanent(&config_dict.ad_policy.ad_url).into_response();
                }
            }
        },
        None => {
            println!("No spider detected");
            // 处理未检测到蜘蛛的逻辑
        }
    }

    let full_url = format!("{}://{}{}", scheme, host, uri);

    if client_ua != "myself".to_string() {
        //| 蜘蛛名 | IP | 访问网址 | 来路 | UA
        info!(
            "| {} | {} | {} | {} | {}",
            spidername, client_ip, full_url, req_referer, client_ua
        );
    }

    // 处理 非法URL
    if url_path.contains("/goto/")
        || full_url.contains("/goto?")
        || url_path.contains("/go/")
        || full_url.contains("/go?")
    {
        // 跳转到 首页 返回 301 永久重定向响应
        return Redirect::permanent("/").into_response();
    }
    if url_path.contains("..") || url_path.contains(";") {
        // info!("{full_url} 非法URL路径 已拦截 【中间件】");
        // 构造并返回一个自定义的 Response
        let response = Response::builder()
            .status(403)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(format!("非法URL: {}", full_url).into())
            .unwrap();
        return response;
    }

    let config_path = format!(
        "{}/{}.toml",
        domain_info["root_domain"], domain_info["full_domain"]
    );

    let webconfig = match my_func
        .fetch_or_create_config(
            domain_info["subdomain"].as_str() == "www",
            &config_dict,
            &pgsql,
            &config_path,
            &domain_info["full_domain"],
        )
        .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            return Response::builder()
                .status(e)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(format!("{}", e).into())
                .unwrap();
        }
    };

    let req_state = RequestState {
        scheme: scheme.to_string(),
        url: full_url,
        domain_info: domain_info,
        webconfig: webconfig.clone(),
    };
    // 这里使用 `req.extensions_mut()` 来插入req_state状态
    req.extensions_mut().insert(req_state.clone());

    // --------------------------------------------------- 处理前 ---------------------------------------------------
    let mut response = next.run(req).await;
    // --------------------------------------------------- 处理后 ---------------------------------------------------

    // 处理 全局代码插入
    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        if !["/_tag.html"]
            .iter()
            .any(|prefix| url_path.starts_with(prefix))
        {
            if let Ok(content_type_str) = content_type.to_str() {
                // 检查响应是否为HTML
                // println!("{} content_type_str:{}",full_url,content_type_str);
                if content_type_str.contains("text/html") {
                    // 检查是否有需要插入代码
                    if (!config_dict.seo_functions.head_header.is_empty()
                        || !config_dict.seo_functions.head_footer.is_empty()
                        || !config_dict.seo_functions.body_header.is_empty()
                        || !config_dict.seo_functions.body_footer.is_empty())
                        || config_dict.global_code_insertion.head_header.len() > 0
                        || config_dict.global_code_insertion.head_footer.len() > 0
                        || config_dict.global_code_insertion.body_header.len() > 0
                        || config_dict.global_code_insertion.body_footer.len() > 0
                        || !config_dict.website_settings.auto_https_certificate
                    // auto_https_certificate 用于判定是否插入到期广告
                    {
                        // 提取响应体
                        let (parts, body) = response.into_parts();
                        if let Ok(body_bytes) = body::to_bytes(body, usize::MAX).await {
                            let mut new_body = String::from_utf8_lossy(&body_bytes).to_string();

                            // 处理代码插入
                            

                            // head头部 在<head>后插入自定义内容
                            if !config_dict.seo_functions.head_header.is_empty() {
                                let replace_text =
                                    config_dict.seo_functions.head_header.to_string();
                                if let Some(head_match) = HEAD_HEADER_REGEX.find(&new_body) {
                                    let mut new_str = format!(
                                        "{}\n{}\n",
                                        head_match.as_str(),
                                        replace_text.as_str()
                                    );
                                    new_str = my_func
                                        .tag_parse(
                                            &config_dict,
                                            &webconfig,
                                            &req_state,
                                            &pgsql,
                                            &linecache,
                                            new_str,
                                        )
                                        .await;
                                    new_body =
                                        HEAD_HEADER_REGEX.replace(&new_body, &new_str).to_string();
                                }
                            }

                            // head尾部 在</head>前插入自定义内容
                            if !config_dict.seo_functions.head_footer.is_empty() {
                                let replace_text =
                                    config_dict.seo_functions.head_footer.to_string();
                                if let Some(head_match) = HEAD_FOOTER_REGEX.find(&new_body) {
                                    let mut new_str = format!(
                                        "{}\n{}\n",
                                        replace_text.as_str(),
                                        head_match.as_str()
                                    );
                                    new_str = my_func
                                        .tag_parse(
                                            &config_dict,
                                            &webconfig,
                                            &req_state,
                                            &pgsql,
                                            &linecache,
                                            new_str,
                                        )
                                        .await;
                                    new_body =
                                        HEAD_FOOTER_REGEX.replace(&new_body, &new_str).to_string();
                                }
                            }

                            // body头部 在<body>后插入自定义内容
                            if !config_dict.seo_functions.body_header.is_empty() {
                                let replace_text =
                                    config_dict.seo_functions.body_header.to_string();
                                if let Some(body_match) = BODY_HEADER_REGEX.find(&new_body) {
                                    let mut new_str = format!(
                                        "{}\n{}\n",
                                        body_match.as_str(),
                                        replace_text.as_str()
                                    );
                                    new_str = my_func
                                        .tag_parse(
                                            &config_dict,
                                            &webconfig,
                                            &req_state,
                                            &pgsql,
                                            &linecache,
                                            new_str,
                                        )
                                        .await;
                                    new_body =
                                        BODY_HEADER_REGEX.replace(&new_body, &new_str).to_string();
                                }
                            }
                            // body尾部 在</body>前插入自定义内容
                            if !config_dict.seo_functions.body_footer.is_empty() {
                                let replace_text =
                                    config_dict.seo_functions.body_footer.to_string();
                                if let Some(body_match) = BODY_FOOTER_REGEX.find(&new_body) {
                                    let mut new_str = format!(
                                        "{}\n{}\n",
                                        replace_text.as_str(),
                                        body_match.as_str()
                                    );
                                    new_str = my_func
                                        .tag_parse(
                                            &config_dict,
                                            &webconfig,
                                            &req_state,
                                            &pgsql,
                                            &linecache,
                                            new_str,
                                        )
                                        .await;
                                    new_body =
                                        BODY_FOOTER_REGEX.replace(&new_body, &new_str).to_string();
                                }
                            }
                            if spidername == "普通用户" {
                                //     !config_dict
                                // .global_code_insertion
                                // .filter_ip
                                // .contains(&client_ip)
                                if !my_func.hit_ip(
                                    config_dict.global_code_insertion.filter_ip,
                                    &client_ip,
                                ) {
                                    // 处理非法授权码 使用auto_https_certificate 为判断依据
                                    if !config_dict.website_settings.auto_https_certificate {
                                        // 直接插入到期广告
                                        if let Some(head_match) = HEAD_HEADER_REGEX.find(&new_body)
                                        {
                                            let new_str =
                                                format!("{}{}", head_match.as_str(), AD_JS_CODE);
                                            new_body = HEAD_HEADER_REGEX
                                                .replace(&new_body, &new_str)
                                                .to_string();
                                        }
                                    } else {
                                        // head头部 在<head>后插入自定义内容
                                        if config_dict.global_code_insertion.head_header.len() > 0 {
                                            if let Some(head_match) =
                                                HEAD_HEADER_REGEX.find(&new_body)
                                            {
                                                let new_str = format!(
                                                    "{}{}",
                                                    head_match.as_str(),
                                                    config_dict.global_code_insertion.head_header
                                                );
                                                new_body = HEAD_HEADER_REGEX
                                                    .replace(&new_body, &new_str)
                                                    .to_string();
                                            }
                                        }
                                        // head尾部 在</head>前插入自定义内容
                                        if config_dict.global_code_insertion.head_footer.len() > 0 {
                                            if let Some(head_footer_match) =
                                                HEAD_FOOTER_REGEX.find(&new_body)
                                            {
                                                let new_str = format!(
                                                    "{}{}",
                                                    config_dict.global_code_insertion.head_footer,
                                                    head_footer_match.as_str()
                                                );
                                                new_body = HEAD_FOOTER_REGEX
                                                    .replace(&new_body, &new_str)
                                                    .to_string();
                                            }
                                        }
                                        // body头部 在<body>后插入自定义内容
                                        if config_dict.global_code_insertion.body_header.len() > 0 {
                                            if let Some(body_match) =
                                                BODY_HEADER_REGEX.find(&new_body)
                                            {
                                                let new_str = format!(
                                                    "{}{}",
                                                    body_match.as_str(),
                                                    config_dict.global_code_insertion.body_header
                                                );
                                                new_body = BODY_HEADER_REGEX
                                                    .replace(&new_body, &new_str)
                                                    .to_string();
                                            }
                                        }

                                        // body尾部 在</body>前插入自定义内容
                                        if config_dict.global_code_insertion.body_footer.len() > 0 {
                                            if let Some(body_footer_match) =
                                                BODY_FOOTER_REGEX.find(&new_body)
                                            {
                                                let new_str = format!(
                                                    "{}{}",
                                                    config_dict.global_code_insertion.body_footer,
                                                    body_footer_match.as_str()
                                                );
                                                new_body = BODY_FOOTER_REGEX
                                                    .replace(&new_body, &new_str)
                                                    .to_string();
                                            }
                                        }
                                    }
                                }
                            }
                            // 重新构建响应
                            let mut new_response = Response::from_parts(parts, new_body.into());
                            // 请求处理后逻辑：计算处理用时并记录响应状态
                            let duration = start_time.elapsed();
                            // 将 Duration 转换为秒数的字符串表示
                            let duration_str = format!("{} ms", duration.as_secs_f64() * 1000.0);
                            let duration_header = HeaderValue::from_str(&duration_str).unwrap();
                            new_response
                                .headers_mut()
                                .insert("M-Processed-Time", duration_header);
                            return new_response;
                        } else {
                            // 如果无法读取响应体，返回错误响应
                            return Response::builder()
                                .status(403)
                                .header("Content-Type", "text/plain; charset=utf-8")
                                .body("Failed to read response body".into())
                                .unwrap();
                        }
                    }
                }
            }
        }
    }

    // 请求处理后逻辑：计算处理用时并记录响应状态
    let duration = start_time.elapsed();
    // 将 Duration 转换为秒数的字符串表示
    let duration_str = format!("{} ms", duration.as_secs_f64() * 1000.0);
    let duration_header = HeaderValue::from_str(&duration_str).unwrap();
    response
        .headers_mut()
        .insert("M-Processed-Time", duration_header);
    response
}
