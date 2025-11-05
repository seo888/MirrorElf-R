use crate::check_webconfig_is_mapping;
use crate::domain_info_from_domain;
// use crate::handlers::website_main::fetch_or_create_config;
// use crate::functions::tag::MTag;
use crate::get_cache_urls;
use crate::get_object_domains;
use crate::get_random_websites;
use crate::my_const::BODY_FOOTER_REGEX;
use crate::my_const::CACHE_PAGE_SUFFIX;
use crate::my_const::HEAD_FOOTER_REGEX;
use crate::AsyncLineCache;
use crate::Config;
use crate::MetaData;
use crate::PgsqlService;
use crate::ReplaceRules;
use crate::RequestState;
use crate::TargetReplaceRules;
use crate::WebsiteConf;
// use crate::functions::minio::MinioClient;
use crate::functions::YouDao;
use crate::my_const::{
    BODY_HEADER_REGEX, CHINA_JSON_PATH, DOC_TAG_REGEX, FIXED_TAG_REGEX, FUNC_TAG_REGEX,
    HEAD_HEADER_REGEX, KUO_HAO_REGEX, SEARCH_URLS, SECRET, SPIDERS_DICT, TITLE_REGEX,
};
use axum::{
    http::{HeaderValue, StatusCode},
    // response::IntoResponse,
};
use encoding_rs::{self, Encoding, UTF_8};
use futures::future::join_all;
use regex::Regex;
// use lazy_static::lazy_static;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
// use minio_rsc::args::{DeleteObjectsArgs, ObjectIdentifier,ListObjectsV2Args};
use chrono::{DateTime, Duration, Local, NaiveDate};
use dashmap::DashMap;
use fake::faker::address::zh_cn::*;
use fake::Fake;
use ip2location_ip2location::bin_format::{Database, TokioFile};
use jwalk::WalkDir;
use minio_rsc::{client::KeyArgs, client::ListObjectsArgs, Minio};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use pinyin::ToPinyin;
use rand::prelude::IndexedRandom;
use rand::seq::IteratorRandom; // <-- This import is required for .choose()
use rand::seq::SliceRandom;
use rand::Rng;
use rand_user_agent::UserAgent;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sqlx::types::chrono::Utc;
use sqlx::Row;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::Command,
    // borrow::Cow,
    rc::Rc,
    sync::Arc,
};
use tldextract_rs::TLDExtract;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use url::Url;
use urlencoding::decode;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct ReplaceRulesYaml {
    pub 全局替换: Vec<String>,
    pub 首页替换: Vec<String>,
    pub 内页替换: Vec<String>,
}

pub struct MyFunc {
    pub ips: Vec<String>,
    pub ipdb: Database<TokioFile>,
    del_comments: Regex,
    del_google: Regex,
    del_author: Regex,
    del_noscript: Regex,
    del_blank_line: Regex,
    re_html_open: Regex,
    re_html_close: Regex,
    re_head_open: Regex,
    re_head_close: Regex,
    re_body_open: Regex,
    re_body_close: Regex,
    re_charset: Regex,
    re_script: Regex,
    re_stream: Regex,
    clean_transed: Regex,
    pub yd: YouDao,
    china_json_data: serde_json::Value,
}

impl MyFunc {
    pub fn new(ips: Vec<String>, ipdb: Database<TokioFile>) -> Self {
        // 1. 读取并解析JSON文件为Value

        let data: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(CHINA_JSON_PATH).expect("Failed to read address data file"),
        )
        .expect("Failed to parse JSON data");
        Self {
            ips,
            ipdb,
            del_comments: Regex::new(r"<!--[\s\S]*?-->").unwrap(),
            del_google: Regex::new(r#"(?i)<meta name="google.*?>"#).unwrap(),
            del_author: Regex::new(r#"(?i)<meta name="author.*?">"#).unwrap(),
            del_noscript: Regex::new(r"(?is)<noscript>.*?</noscript>").unwrap(),
            del_blank_line: Regex::new(r"\n\s*\n").unwrap(),
            re_html_open: Regex::new(r"(?i)<html[\s\S]*?>").unwrap(),
            re_html_close: Regex::new(r"(?i)</html>").unwrap(),
            re_head_open: Regex::new(r"(?i)<head\b[^>]*>").unwrap(),
            re_head_close: Regex::new(r"(?i)</head>").unwrap(),
            re_body_open: Regex::new(r"(?i)<body[\s\S]*?>").unwrap(),
            re_body_close: Regex::new(r"(?i)</body>").unwrap(),
            re_charset: Regex::new(r#"(?i)charset=".*?""#).unwrap(),
            re_script: Regex::new(r"(?is)<script[\s\S]*?>[\s\S]*?</script>").unwrap(),
            re_stream: Regex::new(r"(?:https?:)?//").unwrap(),
            clean_transed: Regex::new(r"<script>var getParameter[\s\S]*</script>").unwrap(),
            yd: YouDao {},
            china_json_data: data,
        }
    }

    pub fn load_replace_string(
        &self,
        replace_string: String,
    ) -> Result<ReplaceRulesYaml, Box<dyn std::error::Error>> {
        //处理无头数据
        if !replace_string.contains("全局替换:") {
            let mut line_string: String;
            if replace_string.len() > 2 {
                // 构造数据
                line_string = "全局替换:\n".to_string();
                if replace_string.contains(" ; ") {
                    for i in replace_string.split(" ; ") {
                        let line = format!("  - '{}'\n", i);
                        line_string.push_str(&line);
                    }
                    line_string.push_str("首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'")
                } else if replace_string.contains("##########") {
                    // 构造数据
                    for i in replace_string.split("##########") {
                        let line = format!("  - '{}'\n", i.replace("----------", " -> "));
                        line_string.push_str(&line);
                    }
                    line_string.push_str("首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'")
                }
            } else {
                line_string = r#"全局替换:
    - '待替换字符串 -> {关键词}'
首页替换:
    - '待替换字符串 -> {关键词2}'
内页替换:
    - '待替换字符串 -> 替换词'"#
                    .to_string();
            }

            // 解析yaml
            let replace_rules: ReplaceRulesYaml = serde_yaml::from_str(&line_string)?;
            return Ok(replace_rules);
        }
        // 解析yaml
        let replace_rules: ReplaceRulesYaml = serde_yaml::from_str(&replace_string)?;
        Ok(replace_rules)
    }

    // 工具函数：将 Vec<String> 转换为 PostgreSQL 数组字面量
    pub fn vec_to_pg_array(vec: &[String]) -> String {
        if vec.is_empty() {
            return "{}".to_string(); // 空数组返回 "{}"
        }

        // 将每个元素转义（如果包含特殊字符，需加双引号）
        let formatted: Vec<String> = vec
            .iter()
            .map(|s| {
                if s.contains(',')
                    || s.contains('{')
                    || s.contains('}')
                    || s.contains('"')
                    || s.contains('\\')
                {
                    format!("\"{}\"", s.replace("\"", "\\\"")) // 转义双引号并包裹
                } else {
                    s.clone() // 无特殊字符，直接使用
                }
            })
            .collect();

        // 用逗号连接并包裹在 {} 中
        format!("{{{}}}", formatted.join(","))
    }

    pub fn get_replace_string(&self, config_re: ReplaceRules) -> String {
        // println!(""config_re.all);
        let mut replace_string = "全局替换:".to_string();
        for i in &config_re.all {
            replace_string.push_str("\n   - '"); // 添加换行符
            replace_string.push_str(&i); // 将 i 转换为 &str
            replace_string.push_str("'")
        }
        replace_string.push_str("\n首页替换:");
        for i in &config_re.index {
            replace_string.push_str("\n   - '"); // 添加换行符
            replace_string.push_str(&i); // 将 i 转换为 &str
            replace_string.push_str("'")
        }
        replace_string.push_str("\n内页替换:");
        for i in &config_re.page {
            replace_string.push_str("\n   - '"); // 添加换行符
            replace_string.push_str(&i); // 将 i 转换为 &str
            replace_string.push_str("'")
        }
        replace_string
    }

    pub fn parse_minio_addr(input: &str) -> HashMap<String, String> {
        // 分割用户名、密码和地址
        let parts: Vec<&str> = input.split('@').collect();
        if parts.len() != 2 {
            panic!("Invalid input format");
        }
        let credentials: Vec<&str> = parts[0].split(':').collect();
        if credentials.len() != 2 {
            panic!("Invalid credentials format");
        }

        // 创建 HashMap 并插入数据
        let mut result = HashMap::new();
        result.insert("username".to_string(), credentials[0].to_string());
        result.insert("password".to_string(), credentials[1].to_string());
        result.insert("address".to_string(), parts[1].to_string());

        result
    }

    /// 获取并过滤 IP 地址
    ///
    /// # 返回值
    /// - `Vec<String>`: 成功返回过滤后的 IP 地址列表，失败返回 `vec!["0.0.0.0"]`
    pub fn get_ips() -> Vec<String> {
        let default_ips = vec!["0.0.0.0".to_string()];

        // 如果文件存在，读取文件内容
        if let Ok(contents) = fs::read_to_string("config/IPS.txt") {
            let ips: Vec<String> = contents
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            if !ips.is_empty() {
                return ips;
            }
        }

        // Execute the command to get IP addresses
        let output = Command::new("sh")
            .arg("-c")
            // .arg("ip addr | grep 'inet ' | awk '{print $2}' | cut -d/ -f1")
            .arg("ip addr | grep 'inet ' | awk '{print $2}' | cut -d/ -f1 | grep -v '^127.0.0.1$' | grep -vE '^10\\.|^172\\.(1[6-9]|2[0-9]|3[0-1])\\.|^192\\.168\\.'")
            .output();

        // Handle command execution error
        let output = match output {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                return default_ips;
            }
        };

        // Convert command output to a string
        let output_str = match String::from_utf8(output.stdout) {
            Ok(output_str) => output_str,
            Err(e) => {
                eprintln!("Failed to convert command output to string: {}", e);
                return default_ips;
            }
        };

        // Split the output into individual IP addresses
        let ips: Vec<String> = output_str
            .trim()
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        // If no IP addresses are found, return the default value
        if ips.is_empty() || ips == [""] {
            default_ips
        } else {
            ips
        }
    }

    pub fn get_random_element(&self, vec: &Vec<String>) -> String {
        if vec.is_empty() {
            return "".to_string();
        }
        let random_index = rand::rng().random_range(0..vec.len());
        // 返回随机元素
        vec[random_index].to_string()
    }

    pub async fn fetch_my_url(url: &str) -> Result<(HeaderValue, Vec<u8>), StatusCode> {
        // 创建 HTTP 客户端，并指定出口 IP
        let client = Client::builder()
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 发送 HTTP 请求
        let response = client
            .get(url)
            .header("User-Agent", "myself") // 设置 User-Agent
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 检查状态码是否为 200 OK
        if response.status() != StatusCode::OK {
            return Err(response.status()); // 返回实际的 HTTP 状态码
        }

        // 获取响应内容类型
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned()
            .unwrap_or_else(|| HeaderValue::from_static("text/html"));

        // 获取响应体字节数据
        let bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .to_vec();

        Ok((content_type, bytes))
    }

    /// 封装网络请求逻辑
    pub async fn fetch_url(&self, url: &str) -> Result<(HeaderValue, Vec<u8>), StatusCode> {
        let rua = UserAgent::pc().to_string();
        let use_ip = self.get_random_element(&self.ips);

        // 解析指定的出口 IP 地址
        let ip: IpAddr = use_ip
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        println!("IP：{} UA：{}", ip, rua);
        // 创建 HTTP 客户端，并指定出口 IP
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // 禁用证书验证
            .local_address(ip) // 设置出口 IP
            .gzip(true)
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 发送 HTTP 请求
        let response = client
            .get(url)
            .header("User-Agent", rua) // 设置 User-Agent
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 检查状态码是否为 200 OK
        if response.status() != StatusCode::OK {
            return Err(response.status()); // 返回实际的 HTTP 状态码
        }

        // 获取响应内容类型
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned()
            .unwrap_or_else(|| HeaderValue::from_static("text/html"));

        // 获取响应体字节数据
        let bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .to_vec();

        Ok((content_type, bytes))
    }

    // 用于获取api json 数据
    pub async fn fetch_url_to_json(&self, url: &str) -> Result<serde_json::Value, StatusCode> {
        let rua = UserAgent::pc().to_string();
        let use_ip = self.get_random_element(&self.ips);

        // 解析指定的出口 IP 地址
        let ip: IpAddr = use_ip
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 创建 HTTP 客户端，并指定出口 IP
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // 禁用证书验证
            .local_address(ip) // 设置出口 IP
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 发送 HTTP 请求
        let response = client
            .get(url)
            .header("User-Agent", rua) // 设置 User-Agent
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 检查状态码是否为 200 OK
        if response.status() != StatusCode::OK {
            return Err(response.status()); // 返回实际的 HTTP 状态码
        }

        // 解析响应体为 JSON
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(json)
    }

    pub fn domain_info_from_domain(&self, domain: &str) -> HashMap<String, String> {
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
                eprintln!("Error extracting domain: {}", e);
                HashMap::new()
            }
        }
    }

    // pub fn domain_info_from_url(&self, url: &str) -> HashMap<String, String> {
    //     println!("get_domain_info url: {}", url);
    //     // 使用 url crate 解析 URL
    //     let parsed_url = Url::parse(url).unwrap();
    //     let domain = parsed_url.host_str().unwrap(); // 获取域名部分
    //     self.domain_info_from_domain(domain)
    // }

    pub fn detect_encoding_and_decode(&self, bytes: &[u8], content_type: &HeaderValue) -> String {
        // 从 Content-Type 头中提取字符集（charset）
        let charset = self.extract_charset_from_content_type(content_type);
        // 获取编码
        let encoding = charset
            .and_then(|name| Encoding::for_label(name.as_bytes()))
            .unwrap_or_else(|| {
                // 使用 encoding_rs 自动检测编码
                if let Some((encoding, _)) = Encoding::for_bom(&bytes) {
                    encoding // 如果检测到 BOM，返回对应的编码
                } else {
                    UTF_8 // 默认回退到 WINDOWS_1252
                }
            });
        // 将字节数据从检测到的编码转换为 UTF-8
        let (text, _, _) = encoding.decode(&bytes);
        text.into_owned()
    }

    /// 从 Content-Type 头中提取字符集（charset）
    fn extract_charset_from_content_type(&self, content_type: &HeaderValue) -> Option<String> {
        content_type
            .to_str()
            .ok()
            .and_then(|header| {
                header
                    .split(';')
                    .find(|part| part.trim().starts_with("charset="))
                    .and_then(|charset| charset.split('=').nth(1))
            })
            .map(|charset| {
                charset
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_lowercase()
            })
    }

    // pub fn clean_transed_html(&self, html_text: &str) -> String {
    //     let source = self.clean_transed.replace(html_text, "").to_string();
    //     let mut output = Vec::new();

    //     // 创建 HTML 重写器
    //     let mut rewriter = HtmlRewriter::new(
    //         Settings {
    //             element_content_handlers: vec![
    //                 // 处理 <a> 标签的 href 属性
    //                 element!("a[href]", |el| {
    //                     if let Some(href) = el.get_attribute("href") {
    //                         if href.trim().len() > 1 {
    //                             // 解码 URL
    //                             if let Some(captures) =
    //                                 Regex::new(r"url=(.*?)&from").unwrap().captures(&href)
    //                             {
    //                                 if let Some(url) = captures.get(1) {
    //                                     let decoded_url = decode(url.as_str())?.to_string();
    //                                     let fully_decoded_url = decode(&decoded_url)?.to_string();
    //                                     el.set_attribute("href", &fully_decoded_url)?;
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     Ok(())
    //                 }),
    //                 // 移除 <base> 标签
    //                 element!("base", |el| {
    //                     el.remove();
    //                     Ok(())
    //                 }),
    //             ],
    //             ..Settings::default()
    //         },
    //         |c: &[u8]| output.extend_from_slice(c),
    //     );

    //     // 处理 HTML 输入
    //     if let Err(err) = rewriter.write(source.as_bytes()) {
    //         eprintln!("Failed to write HTML: {}", err);
    //         return source.to_string(); // 如果出错，返回原始 HTML
    //     }

    //     // 完成处理
    //     if let Err(err) = rewriter.end() {
    //         eprintln!("Failed to end HTML rewriting: {}", err);
    //         return source.to_string(); // 如果出错，返回原始 HTML
    //     }

    //     // 将输出转换为字符串
    //     String::from_utf8(output).unwrap_or_else(|err| {
    //         eprintln!("Failed to convert output to UTF-8: {}", err);
    //         source.to_string() // 如果出错，返回原始 HTML
    //     })
    // }

    pub fn clean_html(&self, html_text: &str, target_domain: &str, trans_mode: bool) -> String {
        let source = if trans_mode {
            self.clean_transed.replace(html_text, "").to_string()
        } else {
            html_text.to_string()
        };

        // println!("title_trans_result:{:?}", r_dict);

        // 过滤 HTML 注释
        let source = self.del_comments.replace_all(&source, "");
        // 过滤 谷歌认证代码
        let source = self.del_google.replace_all(&source, "");
        // 过滤 作者标签
        let source = self.del_author.replace_all(&source, "");
        // 过滤 <noscript> 标签及其内容
        let source = self.del_noscript.replace_all(&source, "");
        // 过滤 空白行
        let source = self.del_blank_line.replace_all(&source, "\n");
        // 格式化 <html> 标签
        let source = self.re_html_open.replace_all(&source, "<html>");
        let source = self.re_html_close.replace_all(&source, "</html>");
        // 格式化 <head> 标签
        let source = self.re_head_open.replace_all(&source, "<head>");
        let source = self.re_head_close.replace_all(&source, "</head>");
        // 格式化 <body> 标签
        let source = self.re_body_open.replace_all(&source, "<body>");
        let source = self.re_body_close.replace_all(&source, "</body>");
        // 替换 charset
        let source = self.re_charset.replace_all(&source, r#"charset="utf-8""#);
        let source = source.replace("gb2312\"", "utf-8\"");
        // 格式化 绝对路径

        let pattern = if target_domain.starts_with("www.") {
            let domain_info = domain_info_from_domain(target_domain);
            format!(
                r#"https?://(?:www\.)?{}/*"#,
                regex::escape(&domain_info["root_domain"])
            )
        } else {
            format!(r#"https?://(?:www\.)?{}/*"#, regex::escape(target_domain))
        };

        let replace_target_re = Regex::new(&pattern).unwrap();
        let source = replace_target_re.replace_all(&source, "/").to_string();
        // let pattern = if target_domain.starts_with("www.") {
        //     let domain_info = domain_info_from_domain(target_domain);
        //     format!(
        //         r#"https?://(?:www\.)?{}[/]?"#,
        //         regex::escape(&domain_info["root_domain"])
        //     )
        // } else {
        //     format!(r#"https?://(?:www\.)?{}[/]?"#, regex::escape(target_domain))
        // };

        // let replace_target_re = Regex::new(&pattern).unwrap();
        // let source = replace_target_re.replace_all(&source, "/").to_string();

        // 检测并修复 <head> 标签（大小写不敏感）
        let source = if !self.re_head_open.is_match(&source) {
            // let re_html_open = Regex::new(r"(?i)<html>").unwrap();
            self.re_html_open
                .replacen(&source, 1, "<html>\n<head>")
                .into_owned()
        } else {
            source
        };
        let mut source = if !self.re_head_close.is_match(&source) {
            // let re_body_open = Regex::new(r"(?i)<body>").unwrap();
            self.re_body_open
                .replacen(&source, 1, "</head>\n<body>")
                .into_owned()
        } else {
            source
        };

        // 正则表达式匹配所有 <script> 标签
        let keywords = [
            "google",
            "elgoog",
            "facebook",
            "koobecaf",
            "cnzz.com",
            "baidu.com",
            "51.la",
        ];
        // 查找所有 <script> 标签
        let scripts: Vec<String> = self
            .re_script
            .find_iter(&source)
            .map(|mat| mat.as_str().to_string())
            .collect();
        // 过滤 包含关键词的 <script> 标签
        for js_code in scripts {
            if keywords.iter().any(|&keyword| js_code.contains(keyword)) {
                source = source.replace(&js_code, "");
            }
        }

        // 处理外部链接 mate link a
        let mut output = Vec::new();
        // 配置 HTML 重写器
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    // 移除 <base> 标签
                    element!("base", |el| {
                        el.remove();
                        Ok(())
                    }),
                    // 删除带有 property 属性的 <meta> 标签
                    element!("meta[property]", |el| {
                        el.remove();
                        Ok(())
                    }),
                    element!("meta[name^='generator']", |el| {
                        el.remove();
                        Ok(())
                    }),
                    element!("meta[name^='author']", |el| {
                        el.remove();
                        Ok(())
                    }),
                    // 处理 <a> 标签的 href 属性
                    element!("a[href]", |el| {
                        if let Some(href) = el.get_attribute("href") {
                            if href.trim().len() > 1 {
                                // 解码 URL
                                if let Some(captures) =
                                    Regex::new(r"url=(.*?)&from").unwrap().captures(&href)
                                {
                                    if let Some(url) = captures.get(1) {
                                        let decoded_url = decode(url.as_str())?.to_string();
                                        let fully_decoded_url = decode(&decoded_url)?.to_string();
                                        el.set_attribute("href", &fully_decoded_url)?;
                                    }
                                }
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 src 属性的 <img> 标签
                    element!("img[src]", |el| {
                        if let Some(img_src) = el.get_attribute("src") {
                            if img_src.starts_with("https://")
                                || img_src.starts_with("http://")
                                || img_src.starts_with("//")
                            {
                                // 图片外链 替换为 流式静态资源链接 /-/
                                let new_img_src = self.re_stream.replace(&img_src, "/-/");
                                el.set_attribute("src", &new_img_src)?;
                            }
                        }
                        Ok(())
                    }),
                    element!("img[srcset]", |el| {
                        if let Some(img_srcset) = el.get_attribute("srcset") {
                            if img_srcset.starts_with("https://")
                                || img_srcset.starts_with("http://")
                                || img_srcset.starts_with("//")
                            {
                                // 图片外链 替换为 流式静态资源链接 /-/
                                let new_img_srcset = self.re_stream.replace_all(&img_srcset, "/-/");
                                el.set_attribute("srcset", &new_img_srcset)?;
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 data-src 属性的 <img> 标签
                    element!("img[data-src]", |el| {
                        if let Some(img_data_src) = el.get_attribute("data-src") {
                            if img_data_src.starts_with("https://")
                                || img_data_src.starts_with("http://")
                                || img_data_src.starts_with("//")
                            {
                                // 图片外链 替换为 流式静态资源链接 /-/
                                let new_img_data_src = self.re_stream.replace(&img_data_src, "/-/");
                                el.set_attribute("data-src", &new_img_data_src)?;
                            }
                        }
                        Ok(())
                    }),
                    element!("img[data-image]", |el| {
                        if let Some(img_data_image) = el.get_attribute("data-image") {
                            if img_data_image.starts_with("https://")
                                || img_data_image.starts_with("http://")
                                || img_data_image.starts_with("//")
                            {
                                // 图片外链 替换为 流式静态资源链接 /-/
                                let new_img_data_image =
                                    self.re_stream.replace(&img_data_image, "/-/");
                                el.set_attribute("data-image", &new_img_data_image)?;
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 href 属性的 <link> 标签
                    element!("link[href]", |el| {
                        if let Some(link_href) = el.get_attribute("href") {
                            if !link_href.contains("googleapis.com") {
                                if link_href.starts_with("https://")
                                    || link_href.starts_with("http://")
                                    || link_href.starts_with("//")
                                {
                                    // link_href 替换为 流式静态资源链接 /-/
                                    let new_link_href = self.re_stream.replace(&link_href, "/-/");
                                    el.set_attribute("href", &new_link_href)?;
                                }
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 src 属性的 <link> 标签
                    element!("link[src]", |el| {
                        if let Some(link_src) = el.get_attribute("src") {
                            if link_src.starts_with("https://")
                                || link_src.starts_with("http://")
                                || link_src.starts_with("//")
                            {
                                // link_src 替换为 流式静态资源链接 /-/
                                let new_link_src = self.re_stream.replace(&link_src, "/-/");
                                el.set_attribute("src", &new_link_src)?;
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 srcset 属性的 <source> 标签
                    element!("source[srcset]", |el| {
                        if let Some(source_srcset) = el.get_attribute("srcset") {
                            if source_srcset.starts_with("https://")
                                || source_srcset.starts_with("http://")
                                || source_srcset.starts_with("//")
                            {
                                // source_srcset 替换为 流式静态资源链接 /-/
                                let new_source_srcset =
                                    self.re_stream.replace_all(&source_srcset, "/-/");
                                el.set_attribute("srcset", &new_source_srcset)?;
                            }
                        }
                        Ok(())
                    }),
                    // 处理带有 data-srcset 属性的 <source> 标签
                    element!("source[data-srcset]", |el| {
                        if let Some(source_data_srcset) = el.get_attribute("data-srcset") {
                            if source_data_srcset.starts_with("https://")
                                || source_data_srcset.starts_with("http://")
                                || source_data_srcset.starts_with("//")
                            {
                                // source_data_srcset 替换为 流式静态资源链接 /-/
                                let new_source_data_srcset =
                                    self.re_stream.replace_all(&source_data_srcset, "/-/");
                                el.set_attribute("data-srcset", &new_source_data_srcset)?;
                            }
                        }
                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c), // 输出回调
        );

        // 处理 HTML 输入
        if let Err(err) = rewriter.write(source.as_bytes()) {
            eprintln!("Failed to write HTML: {}", err);
            return html_text.to_string(); // 如果出错，返回原始 HTML
        }

        // 完成处理
        if let Err(err) = rewriter.end() {
            eprintln!("Failed to end HTML rewriting: {}", err);
            return html_text.to_string(); // 如果出错，返回原始 HTML
        }

        // 将输出转换为字符串
        let source = String::from_utf8(output).unwrap_or_else(|err| {
            eprintln!("Failed to convert output to UTF-8: {}", err);
            html_text.to_string() // 如果出错，返回原始 HTML
        });

        if trans_mode {
            // 格式化 绝对路径
            // let pattern = format!(r#"https?://(?:www\.)?{}[/]?"#, regex::escape(target_domain));
            let pattern = if target_domain.starts_with("www.") {
                let domain_info = domain_info_from_domain(target_domain);
                format!(
                    r#"https?://(?:www\.)?{}[/]?"#,
                    regex::escape(&domain_info["root_domain"])
                )
            } else {
                format!(r#"https?://(?:www\.)?{}[/]?"#, regex::escape(target_domain))
            };
            let replace_target_re = Regex::new(&pattern).unwrap();
            let source = replace_target_re.replace_all(&source, "/").to_string();
            return source;
        }
        source
    }

    // 处理标签 ${}
    fn replace_tags(&self, text: &str, webconfig: &WebsiteConf) -> String {
        // 处理 keywords
        let keywords_vec: Vec<&str> = webconfig.info.keywords.split(",").collect();
        if keywords_vec.is_empty() {
            return text.to_string(); // 如果没有关键词，直接返回
        }
        let keyword1 = keywords_vec.get(0).unwrap_or(&"");
        let mut result = text.to_string();

        // 正则匹配：{keyword数字}、{关键词数字}、【关键词数字】
        let re = Regex::new(r"\{keyword(\d+)\}|\{关键词(\d+)\}|【关键词(\d+)】").unwrap();

        // 替换所有带数字的占位符
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                // 提取数字部分（可能来自 ${keyword}、${关键词}、{keyword}、{关键词}、【关键词】）
                let num_str = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .or_else(|| caps.get(3))
                    .or_else(|| caps.get(4))
                    .or_else(|| caps.get(5))
                    .or_else(|| caps.get(6))
                    .or_else(|| caps.get(7))
                    .or_else(|| caps.get(8))
                    .or_else(|| caps.get(9))
                    .unwrap()
                    .as_str();
                let num = num_str.parse::<usize>().unwrap_or(1); // 默认 1 如果解析失败

                // 获取对应的关键词（如果超出范围，使用 keyword1）
                keywords_vec.get(num.saturating_sub(1)).unwrap_or(keyword1)
            })
            .to_string();

        // 处理无数字的情况（${keyword}、${关键词}、{keyword}、{关键词}、【关键词】）
        result = result
            // .replace("${keyword}", keyword1)
            // .replace("${关键词}", keyword1)
            // .replace("{keyword}", keyword1)
            // .replace("{关键词}", keyword1)
            .replace("【关键词】", "{关键词}");

        // // 处理title
        // let re = Regex::new(r"\$\{title\}|\$\{标题\}|\{title\}|\{标题\}|【标题】").unwrap();
        // result = re
        //     .replace_all(&result, |_caps: &regex::Captures| &webconfig.info.title)
        //     .to_string();
        // // 处理description
        // let re =
        //     Regex::new(r"\$\{description\}|\$\{描述\}|\{description\}|\{描述\}|【描述】").unwrap();
        // result = re
        //     .replace_all(&result, |_caps: &regex::Captures| {
        //         &webconfig.info.description
        //     })
        //     .to_string();

        result
    }
    // fn replace_var(&self, text: &str, keywords_vec: &Vec<&str>) -> String {
    //     // 初始化 keyword1, keyword2, keyword3
    //     let keyword1 = keywords_vec.get(0).unwrap_or(&"");
    //     let keyword2 = keywords_vec.get(1).unwrap_or(keyword1);
    //     let keyword3 = keywords_vec.get(2).unwrap_or(keyword1);
    //     let keyword4 = keywords_vec.get(3).unwrap_or(keyword1);
    //     let keyword5 = keywords_vec.get(4).unwrap_or(keyword1);

    //     // 检查是否需要替换
    //     if !text.contains("{keyword") && !text.contains("{关键词") && !text.contains("【关键词") {
    //         return text.to_string(); // 如果没有占位符，直接返回原文本
    //     }

    //     // 替换占位符
    //     text.replace("{keyword}", keyword1)
    //         .replace("{keyword1}", keyword1)
    //         .replace("{keyword2}", keyword2)
    //         .replace("{keyword3}", keyword3)
    //         .replace("{keyword4}", keyword4)
    //         .replace("{keyword5}", keyword5)
    //         .replace("{关键词}", keyword1)
    //         .replace("{关键词1}", keyword1)
    //         .replace("{关键词2}", keyword2)
    //         .replace("{关键词3}", keyword3)
    //         .replace("{关键词4}", keyword4)
    //         .replace("{关键词5}", keyword5)
    //         .replace("【关键词】", keyword1)
    //         .replace("【关键词1】", keyword1)
    //         .replace("【关键词2】", keyword2)
    //         .replace("【关键词3】", keyword3)
    //         .replace("【关键词4】", keyword4)
    //         .replace("【关键词5】", keyword5)
    // }

    // 清理路径（去掉 # 及其后面的部分）
    fn link_clean(&self, path: &str) -> String {
        path.split('#').next().unwrap_or("").to_string()
    }

    // 反转路径
    fn reverse_path(&self, url: &str) -> String {
        let (path, params) = match url.split_once('?') {
            Some((path, query)) => (path, query),
            None => (url, ""),
        };

        // 去除 path 前后的 /
        let path = path.trim_matches('/');

        // 不再需要手动添加前导 /，因为后续会统一处理
        let (path_without_file, file_part) =
            if let Some((path_part, file_part)) = path.rsplit_once('/') {
                (path_part, file_part)
            } else {
                ("", path)
            };

        let (file_name, suffix) = file_part.split_once('.').unwrap_or((file_part, ""));

        // 反转文件名前部分
        let reversed_file_name = file_name.chars().rev().collect::<String>();

        // 反转路径部分
        let parts: Vec<&str> = path_without_file
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let reversed_path = if parts.is_empty() {
            "".to_string()
        } else if parts.len() == 1 {
            parts[0].chars().rev().collect::<String>()
        } else {
            parts.iter().rev().copied().collect::<Vec<_>>().join("/")
        };

        // 构造最终路径，确保后缀被正确附加
        let final_path = if !reversed_file_name.is_empty() && !suffix.is_empty() {
            if reversed_path.is_empty() {
                format!("/{}.{}", reversed_file_name, suffix)
            } else {
                format!("/{}/{}.{}", reversed_path, reversed_file_name, suffix)
            }
        } else if !reversed_file_name.is_empty() {
            // 修复点：当 reversed_path 为空时直接使用文件名
            if reversed_path.is_empty() {
                format!("/{}", reversed_file_name)
            } else {
                format!("/{}/{}", reversed_path, reversed_file_name)
            }
        } else {
            format!("/{}", reversed_path)
        };

        if params.is_empty() {
            final_path
        } else {
            format!("{}?{}", final_path, params)
        }
    }

    // 路径映射
    fn link_mapping(&self, path: &str) -> (String, String) {
        let path = self.link_clean(path);
        // 解码 URL
        let decoded_href = decode(&path).unwrap_or_default();

        // 检查是否包含中文字符
        let is_chinese = Regex::new(r"[\u4e00-\u9fff]")
            .unwrap()
            .is_match(&decoded_href);

        let new_path = if is_chinese {
            // 将中文字符转换为拼音
            let mut pinyin_string = String::new();
            for c in decoded_href.chars() {
                // 检查是否为中文字符（这里假设使用 Unicode 范围来判断）
                if ('\u{4e00}'..='\u{9fff}').contains(&c) {
                    // 如果是中文字符，尝试转换为拼音
                    if let Some(pinyin) = c.to_pinyin() {
                        let result = pinyin.plain().replace("ü", "v");
                        pinyin_string.push_str(&result);
                    }
                } else {
                    // 如果不是中文字符，直接保留原样
                    pinyin_string.push(c);
                }
            }
            pinyin_string
        } else {
            // 翻转路径
            self.reverse_path(&decoded_href)
        };

        (path.to_string(), new_path)
    }

    fn re_or_replace(&self, text: &str, left: &str, right: &str) -> String {
        if left.starts_with("r#") {
            // 提取正则表达式部分
            let pattern = &left[2..];
            match Regex::new(pattern) {
                Ok(re) => {
                    // 使用正则替换
                    re.replace_all(text, right).to_string()
                }
                Err(_) => {
                    // 如果正则表达式无效，退回到普通字符串替换
                    text.replace(&left[2..], right)
                }
            }
        } else {
            // 使用普通字符串替换
            text.replace(left, right)
        }
    }

    fn replacer(&self, html_text: &str, keywords: &str, rules: &Vec<String>) -> String {
        let mut text = html_text.to_string();
        let keywords_vec: Vec<&str> = keywords.split(",").collect();
        for re_line in rules {
            if re_line.contains("----------") || re_line.contains("##########") {
                // 单行替换字符格式
                let line_re_list: Vec<&str> = re_line.split("##########").collect();
                for line in line_re_list {
                    if let Some((left, right)) = line.split_once("----------") {
                        // let right = self.replace_var(right, &keywords_vec);
                        // text = text.replace(left, &right);
                        text = self.re_or_replace(&text, left, &right);
                    } else {
                        println!("分隔符未找到 直接替换为核心词");
                        if let Some(first_keyword) = keywords_vec.first() {
                            // text = text.replace(line, first_keyword);
                            text = self.re_or_replace(&text, line, first_keyword);
                        }
                    }
                }
            } else {
                // " -> " 替换字符格式
                if let Some((left, right)) = re_line.split_once(" -> ") {
                    // let right = self.replace_var(right, &keywords_vec);
                    // text = text.replace(left, &right);
                    text = self.re_or_replace(&text, left, &right);
                } else {
                    println!("分隔符未找到 直接替换为核心词");
                    if let Some(first_keyword) = keywords_vec.first() {
                        // text = text.replace(re_line, first_keyword);
                        text = self.re_or_replace(&text, re_line, first_keyword);
                    }
                }
            }
        }
        text
    }

    pub fn get_target_lang_domain(target: &str) -> (String, String) {
        if target.contains('|') {
            let mut parts = target.split('|');
            let lang = parts.next().unwrap_or("").to_string(); // 获取语言部分
            let domain = parts.next().unwrap_or("").to_string(); // 获取域名部分
            (lang, domain)
        } else {
            // 如果没有 |，则默认语言为空，域名是 target_domain
            ("zh".to_string(), target.to_string())
        }
    }

    // 缓存前替换处理
    pub async fn replace_html(
        &self,
        html_text: &str,
        is_index: bool,
        target_re: &TargetReplaceRules,
        webconfig: &WebsiteConf,
        config_dict: &Config,
        req_state: &RequestState,
        linecache: &Arc<AsyncLineCache>,
    ) -> (String, HashMap<String, String>) {
        let url = req_state.url.clone();
        let domain = req_state.domain_info["full_domain"].to_string();
        let root_domain = req_state.domain_info["root_domain"].to_string();

        let mut source = html_text.to_string();
        // println!("webconfig: {:?}", webconfig);
        // println!("target_re: {:?}", target_re);
        match webconfig.re.replace_mode {
            1 => {
                // 1: 先 目标站替换 后 本站替换
                // 处理目标站替换
                if is_index && !target_re.index.is_empty() {
                    source = self.replacer(&html_text, &webconfig.info.keywords, &target_re.index);
                }
                if !is_index && !target_re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.page);
                }
                if !target_re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.all);
                }
                // 处理本站替换
                if is_index && !webconfig.re.index.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.index);
                }
                if !is_index && !webconfig.re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.page);
                }
                if !webconfig.re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.all);
                }
            }
            2 => {
                // 2: 仅 本站替换
                // 处理本站替换
                if is_index && !webconfig.re.index.is_empty() {
                    source =
                        self.replacer(&html_text, &webconfig.info.keywords, &webconfig.re.index);
                }
                if !is_index && !webconfig.re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.page);
                }
                if !webconfig.re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.all);
                }
            }
            3 => {
                // 3: 先 本站替换 后 目标站替换
                // 处理本站替换
                if is_index && !webconfig.re.index.is_empty() {
                    source =
                        self.replacer(&html_text, &webconfig.info.keywords, &webconfig.re.index);
                }
                if !is_index && !webconfig.re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.page);
                }
                if !webconfig.re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &webconfig.re.all);
                }
                // 处理目标站替换
                if is_index && !target_re.index.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.index);
                }
                if !is_index && !target_re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.page);
                }
                if !target_re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.all);
                }
            }
            _ => {
                // 0: 仅 目标站替换
                // 处理目标站替换
                if is_index && !target_re.index.is_empty() {
                    source = self.replacer(&html_text, &webconfig.info.keywords, &target_re.index);
                }
                if !is_index && !target_re.page.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.page);
                }
                if !target_re.all.is_empty() {
                    source = self.replacer(&source, &webconfig.info.keywords, &target_re.all);
                }
            }
        }

        // 处理标签 还有所有自定义标签 &{} 一起处理
        source = self.replace_tags(&source, &webconfig);
        // let cache_page_suffix = [".php", ".asp", ".jsp", ".html", ".htm", ".shtml"];

        let page_title = TITLE_REGEX
            .captures(&source)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or(&webconfig.info.title) // 如果没有找到 title 标签，使用空字符串作为默认值
            .to_string();

        // 处理外部链接 mate link a
        let internal_links = RefCell::new(HashMap::new());
        let mut output = Vec::new();
        // 配置 HTML 重写器
        let mut rewriter;
        let class_handler = element!("*", |el| {
            // "*" 匹配所有元素
            if config_dict.seo_functions.random_class_name {
                if let Some(old_class) = el.get_attribute("class") {
                    let new_class = format!("{} {}", Self::generate_random_class(), old_class);
                    el.set_attribute("class", &new_class).unwrap();
                }
            }
            Ok(())
        });
        // 定义 <a> 标签处理器
        let a_handler = element!("a[href]", |el| {
            if let Some(href) = el.get_attribute("href") {
                if href.starts_with("//")
                    || href.starts_with("http://")
                    || href.starts_with("https://")
                {
                    // 外部链接
                    let contains_any = config_dict
                        .seo_functions
                        .external_filter
                        .iter()
                        .any(|ban_suffix| href.contains(ban_suffix));
                    if !contains_any {
                        el.set_attribute("href", "【new_link】").unwrap();
                    }
                } else {
                    if (!href.contains(".")
                        || CACHE_PAGE_SUFFIX.iter().any(|suffix| href.contains(suffix)))
                        && !href.contains("(")
                        && !href.contains(";")
                        && !href.starts_with("#")
                    {
                        if webconfig.info.link_mapping {
                            // 链接映射
                            let (link_path, mapping_path) = self.link_mapping(&href);
                            // 更新 href 属性
                            el.set_attribute("href", &mapping_path).unwrap();
                            // 写入字典
                            internal_links.borrow_mut().insert(link_path, mapping_path);
                        } else {
                            let link_path = self.link_clean(&href);
                            internal_links
                                .borrow_mut()
                                .insert(link_path, "".to_string());
                        }
                    }
                }
            }
            Ok(())
        });
        let div_handler = element!("div", |el| {
            if config_dict.seo_functions.random_div_attributes {
                // 添加当前时间戳作为 date-time 属性
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                el.set_attribute("date-time", &timestamp.to_string())?;
                // 添加 dir 属性
                let random_dir: String = (0..6)
                    .map(|_| (rand::random::<u8>() % 26 + b'a') as char)
                    .collect();
                el.set_attribute("dir", &random_dir)?;
            }
            Ok(())
        });
        // 匹配所有元素，检查是否有 class 属性
        // let class_handler = element!("*", |el| {
        //     if let Some(old_class) = el.get_attribute("class") {
        //         let new_class = format!("{} {}", old_class, Self::generate_random_class());
        //         el.set_attribute("class", &new_class).unwrap();
        //     }
        //     Ok(())
        // });
        // let class_handler = element!("a[href]", |el| {
        //     if let Some(old_class) = el.get_attribute("class") {
        //         let new_class = format!("{} {}", old_class, Self::generate_random_class());
        //         el.set_attribute("class", &new_class).unwrap();
        //     }
        //     Ok(())
        // });

        if is_index {
            // 首页模式
            rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![
                        class_handler,
                        a_handler,
                        div_handler,
                        element!("title", |el| {
                            el.remove(); // 直接移除整个元素
                            Ok(())
                        }),
                        element!("meta[name='description' i]", |el| {
                            el.remove(); // 直接移除整个元素
                            Ok(())
                        }),
                        element!("meta[name='keywords' i]", |el| {
                            el.remove(); // 直接移除整个元素
                            Ok(())
                        }),
                        // element!("title", |el| {
                        //     el.set_inner_content(&webconfig.info.title, ContentType::Text);
                        //     Ok(())
                        // }),
                        // // Replace <meta name="description"> tag content
                        // element!("meta[name='description']", |el| {
                        //     if let Some(_) = el.get_attribute("content") {
                        //         el.set_attribute("content", &webconfig.info.description)?;
                        //     }
                        //     Ok(())
                        // }),
                        // // Replace <meta name="keywords"> tag content
                        // element!("meta[name='keywords']", |el| {
                        //     if let Some(_) = el.get_attribute("content") {
                        //         el.set_attribute("content", &webconfig.info.keywords)?;
                        //     }
                        //     Ok(())
                        // }),

                        // 在 <head> 结束前添加 meta 标签（如果需要）
                        element!("head", |el| {
                            let first_keyword = webconfig
                                .info
                                .keywords
                                .split(',')
                                .next()
                                .unwrap_or("")
                                .trim();
                            el.append(
                                &format!("<title>{}</title>\n", &webconfig.info.title),
                                ContentType::Html,
                            );

                            el.append(
                                &format!(
                                    "<meta name=\"keywords\" content=\"{}\">\n",
                                    &webconfig.info.keywords
                                ),
                                ContentType::Html,
                            );
                            el.append(
                                &format!(
                                    "<meta name=\"description\" content=\"{}\">\n",
                                    &webconfig.info.description
                                ),
                                ContentType::Html,
                            );
                            el.append(
                                &format!("<meta name=\"author\" content=\"{}\">\n", &first_keyword),
                                ContentType::Html,
                            );

                            if config_dict.seo_functions.meta_information {
                                // <meta property="og:url" content="https://www.zh-ios-fifaclub.com" />
                                el.append(
                                    &format!("<meta property=\"og:url\" content=\"{}\">\n", &url),
                                    ContentType::Html,
                                );
                                // <meta property="og:type" content="website" />
                                el.append(
                                    &format!(
                                        "<meta property=\"og:type\" content=\"{}\">\n",
                                        "website"
                                    ),
                                    ContentType::Html,
                                );
                                // <meta property="og:site_name" content="世俱杯下注(买球)官方网站-2025 club world cup" />
                                el.append(
                                    &format!(
                                        "<meta property=\"og:site_name\" content=\"{}\">\n",
                                        first_keyword
                                    ),
                                    ContentType::Html,
                                );
                                // <meta property="og:title" content="世俱杯下注(买球)官方网站-2025 club world cup" />
                                el.append(
                                    &format!(
                                        "<meta property=\"og:title\" content=\"{}\">\n",
                                        &webconfig.info.title
                                    ),
                                    ContentType::Html,
                                );
                                // <meta name="og:description" content="世俱杯买球提供实时更新的赛事赔率与丰富玩法选择，满足不同玩家的投注需求。作为正规世俱杯下注平台，我们采用多重加密技术确保交易安全，支持多种主流支付方式。新用户注册即享专属优惠，体验流畅稳定的在线投注服务。" />
                                el.append(
                                    &format!(
                                        "<meta property=\"og:keywords\" content=\"{}\">\n",
                                        &webconfig.info.keywords
                                    ),
                                    ContentType::Html,
                                );
                                el.append(
                                    &format!(
                                        "<meta property=\"og:description\" content=\"{}\">\n",
                                        &webconfig.info.description
                                    ),
                                    ContentType::Html,
                                );
                                // <meta name="twitter:site" content="https://www.zh-ios-fifaclub.com" />
                                el.append(
                                    &format!("<meta name=\"twitter:site\" content=\"{}\">\n", &url),
                                    ContentType::Html,
                                );
                                // <meta name="twitter:title" content="世俱杯下注(买球)官方网站-2025 club world cup" />
                                el.append(
                                    &format!(
                                        "<meta name=\"twitter:title\" content=\"{}\">\n",
                                        &webconfig.info.title
                                    ),
                                    ContentType::Html,
                                );
                                // <meta name="twitter:description" content="世俱杯买球提供实时更新的赛事赔率与丰富玩法选择，满足不同玩家的投注需求。作为正规世俱杯下注平台，我们采用多重加密技术确保交易安全，支持多种主流支付方式。新用户注册即享专属优惠，体验流畅稳定的在线投注服务。" />
                                el.append(
                                    &format!(
                                        "<meta name=\"twitter:description\" content=\"{}\">\n",
                                        &webconfig.info.description
                                    ),
                                    ContentType::Html,
                                );
                                let address = self.chinese_address();
                                let script = format!(
                                    r#"<script type="application/ld+json">
{{
  "@context": "https://schema.org",
  "@type": "Organization",
  "name": "{}",
  "url": "{}",
  "keywords": "{}",
  "description": "{}",
  "logo": "http://{}/favicon.ico",
  "sameAs": [
    "https://www.bing.com/{}",
    "https://cn.bing.com/{}",
    "https://www.google.com/{}",
    "https://www.facebook.com/{}",
    "https://www.linkedin.com/company/{}",
    "https://twitter.com/{}"
  ],
  "contactPoint": [
    {{
      "@type": "ContactPoint",
      "contactType": "customer service",
      "telephone": "{}",
      "email": "service@{}",
      "areaServed": "CN",
      "availableLanguage": ["Chinese", "English"],
      "contactOption": "TollFree",
      "faxNumber": "{}"
                            }}
  ],
  "address": {{
    "@type": "PostalAddress",
    "streetAddress": "{}",
    "postalCode": "{}",
    "addressCountry": "CN"
                            }}
}}</script>"#,
                                    &webconfig.info.title,
                                    &url,
                                    &webconfig.info.keywords,
                                    &webconfig.info.description,
                                    domain,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    Self::chinese_phone_number(),
                                    root_domain,
                                    Self::chinese_fax_number(),
                                    address,
                                    Self::chinese_postal_code(&address),
                                );
                                el.append(&script, ContentType::Html);
                            }
                            Ok(())
                        }),
                    ],
                    ..Settings::default()
                },
                Box::new(|c: &[u8]| output.extend_from_slice(c)) as Box<dyn FnMut(&[u8])>, // Box the closure
            );
        } else {
            // 内页模式
            // println!("内页替换TDK，预留给自定义设置");
            rewriter = HtmlRewriter::new(
                Settings {
                    element_content_handlers: vec![
                        class_handler,
                        a_handler,
                        div_handler,
                        element!("meta[name='keywords' i]", |el| {
                            if let Some(content) = el.get_attribute("content") {
                                let new_content =
                                    format!("{},{}", content, &webconfig.info.keywords);
                                el.set_attribute("content", &new_content)?;
                            } else {
                                el.set_attribute("content", &webconfig.info.keywords)?;
                            }
                            Ok(())
                        }),
                        element!("head", |el| {
                            let first_keyword = webconfig
                                .info
                                .keywords
                                .split(',')
                                .next()
                                .unwrap_or("")
                                .trim();
                            el.append(
                                &format!("<meta name=\"author\" content=\"{}\">\n", &first_keyword),
                                ContentType::Html,
                            );
                            if config_dict.seo_functions.meta_information {
                                el.append(
                                    &format!("<meta property=\"og:url\" content=\"{}\">\n", &url),
                                    ContentType::Html,
                                );
                                el.append(
                                    &format!(
                                        "<meta property=\"og:type\" content=\"{}\">\n",
                                        "article"
                                    ),
                                    ContentType::Html,
                                );
                                el.append(
                                    &format!(
                                        "<meta property=\"og:site_name\" content=\"{}\">\n",
                                        &webconfig.info.title
                                    ),
                                    ContentType::Html,
                                );
                                el.append(
                                    &format!(
                                        "<meta property=\"og:title\" content=\"{}\">\n",
                                        &page_title
                                    ),
                                    ContentType::Html,
                                );
                                el.append(
                                    &format!(
                                        "<meta property=\"og:keywords\" content=\"{}\">\n",
                                        &webconfig.info.keywords
                                    ),
                                    ContentType::Html,
                                );
                                let script = format!(
                                    r#"<script type="application/ld+json">
{{
  "@context": "https://schema.org",
  "@type": "Organization",
  "name": "{}",
  "url": "{}",
  "keywords": "{}",
  "description": "{}",
  "logo": "http://{}/favicon.ico",
  "sameAs": [
    "https://www.bing.com/{}",
    "https://cn.bing.com/{}",
    "https://www.google.com/{}",
    "https://www.facebook.com/{}",
    "https://www.linkedin.com/company/{}",
    "https://twitter.com/{}"
  ]
}}</script>"#,
                                    &webconfig.info.title,
                                    &url,
                                    &webconfig.info.keywords,
                                    &webconfig.info.description,
                                    domain,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword,
                                    first_keyword
                                );
                                el.append(&script, ContentType::Html);
                            }
                            Ok(())
                        }),
                    ],
                    ..Settings::default()
                },
                Box::new(|c: &[u8]| output.extend_from_slice(c)) as Box<dyn FnMut(&[u8])>, // Box the closure
            );
        }

        // let links = internal_links.into_inner();

        // 处理 HTML 输入
        if let Err(err) = rewriter.write(source.as_bytes()) {
            eprintln!("Failed to write HTML: {}", err);
            return (html_text.to_string(), HashMap::new()); // 如果出错，返回原始 HTML
        }

        // 完成处理
        if let Err(err) = rewriter.end() {
            eprintln!("Failed to end HTML rewriting: {}", err);
            return (html_text.to_string(), HashMap::new()); // 如果出错，返回原始 HTML
        }

        let mut new_html = String::from_utf8(output).unwrap_or_else(|err| {
            eprintln!("Failed to convert output to UTF-8: {}", err);
            html_text.to_string()
        });

        // 处理 h1 seo
        // if !config_dict.seo_functions.h1_seo.is_empty() {
        //     let h1 = config_dict.seo_functions.h1_seo.to_string();
        //     let new_h1 = h1;
        //     if let Some(body_match) = BODY_HEADER_REGEX.find(&new_html) {
        //         let new_str = format!("{}\n{}\n", body_match.as_str(), new_h1.as_str());
        //         new_html = BODY_HEADER_REGEX.replace(&new_html, &new_str).to_string();
        //     }
        // }

        // // head头部 在<head>后插入自定义内容
        // if !config_dict.seo_functions.head_header.is_empty() {
        //     let replace_text = config_dict.seo_functions.head_header.to_string();
        //     if let Some(head_match) = HEAD_HEADER_REGEX.find(&new_html) {
        //         let new_str = format!("{}\n{}\n", head_match.as_str(), replace_text.as_str());
        //         new_html = HEAD_HEADER_REGEX.replace(&new_html, &new_str).to_string();
        //     }
        // }

        // // head尾部 在</head>前插入自定义内容
        // if !config_dict.seo_functions.head_footer.is_empty() {
        //     let replace_text = config_dict.seo_functions.head_footer.to_string();
        //     if let Some(head_match) = HEAD_FOOTER_REGEX.find(&new_html) {
        //         let new_str = format!("{}\n{}\n", head_match.as_str(), replace_text.as_str());
        //         new_html = HEAD_FOOTER_REGEX.replace(&new_html, &new_str).to_string();
        //     }
        // }

        // // body头部 在<body>后插入自定义内容
        // if !config_dict.seo_functions.body_header.is_empty() {
        //     let replace_text = config_dict.seo_functions.body_header.to_string();
        //     if let Some(body_match) = BODY_HEADER_REGEX.find(&new_html) {
        //         let new_str = format!("{}\n{}\n", body_match.as_str(), replace_text.as_str());
        //         new_html = BODY_HEADER_REGEX.replace(&new_html, &new_str).to_string();
        //     }
        // }
        // // body尾部 在</body>前插入自定义内容
        // if !config_dict.seo_functions.body_footer.is_empty() {
        //     let replace_text = config_dict.seo_functions.body_footer.to_string();
        //     if let Some(body_match) = BODY_FOOTER_REGEX.find(&new_html) {
        //         let new_str = format!("{}\n{}\n", body_match.as_str(), replace_text.as_str());
        //         new_html = BODY_FOOTER_REGEX.replace(&new_html, &new_str).to_string();
        //     }
        // }

        // 处理目标网址链接
        let (target_lang, target_domain) =
            MyFunc::get_target_lang_domain(webconfig.info.target.as_str());
        new_html = MyFunc::replace_domain(&new_html, &target_domain, &domain);

        (new_html, internal_links.into_inner())
    }

    fn generate_random_class() -> String {
        rand::rng()
            .random_range(1_000_000_000i64..=9_999_999_999i64)
            .to_string()
    }

    fn replace_domain(input: &str, target_domain: &str, replacement: &str) -> String {
        // 转义目标域名以处理特殊字符
        let escaped_domain = regex::escape(target_domain);

        // 使用单词边界匹配独立域名，大小写不敏感
        let pattern = format!(r"(?i)\b{}\b", escaped_domain);
        let re = Regex::new(&pattern).expect("无法编译正则表达式");

        re.replace_all(input, |caps: &regex::Captures| {
            let matched_str = caps.get(0).unwrap().as_str();
            let match_start = caps.get(0).unwrap().start();

            // 检查匹配是否被点号（.）前缀（例如子域名）
            if match_start > 0 && input.chars().nth(match_start - 1) == Some('.') {
                // 保留原匹配字符串（例如 sub.gdstc.gd.gov.cn）
                matched_str.to_string()
            } else {
                // 替换为新域名
                replacement.to_string()
            }
        })
        .to_string()
    }

    fn chinese_phone_number() -> String {
        let mut rng = rand::rng();

        // 常见号段
        let prefixes = ["13", "15", "18", "170", "171"];

        let prefix = prefixes[rng.random_range(0..prefixes.len())];
        let mut phone = String::from(prefix);

        // 补全剩余位数
        for _ in 0..(11 - prefix.len()) {
            phone.push(rng.random_range(0..10).to_string().chars().next().unwrap());
        }

        phone
    }

    fn chinese_fax_number() -> String {
        let mut rng = rand::rng();

        // 常见区号（包含直辖市和其他主要城市）
        let area_codes = [
            "010", "021", "022", "023", // 直辖市
            "024", "025", "027", "028", "029", // 其他省会
            "0755", "0731", "0512", // 深圳、长沙、苏州等
        ];

        // 随机选择区号
        let area_code = area_codes[rng.random_range(0..area_codes.len())];

        // 生成本地号码（6-8位）
        let local_number_length = rng.random_range(6..=8);
        let mut local_number = String::new();
        for _ in 0..local_number_length {
            local_number.push(rng.random_range(0..10).to_string().chars().next().unwrap());
        }

        format!("{}-{}", area_code, local_number)
    }

    /// 从JSON文件随机生成中国地址（简化版）
    pub fn chinese_address(&self) -> String {
        let mut rng = rand::rng();
        // 2. 随机选择省
        let provinces = self
            .china_json_data
            .as_object()
            .expect("Root should be an object");
        let (province, cities) = provinces
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .choose(&mut rng)
            .unwrap()
            .to_owned();

        // 3. 随机选择市
        let cities_map = cities.as_object().expect("Cities should be an object");
        let (city, districts) = cities_map
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .choose(&mut rng)
            .unwrap()
            .to_owned();

        // 4. 随机选择区
        let districts_map = districts
            .as_object()
            .expect("Districts should be an object");
        let (district, streets) = districts_map
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .choose(&mut rng)
            .unwrap()
            .to_owned();

        // 5. 处理街道信息
        let street = if let Some(streets_map) = streets.as_object() {
            streets_map
                .keys()
                .collect::<Vec<_>>()
                .iter()
                .choose(&mut rng)
                .map(|s| s.as_str())
                .unwrap_or("")
        } else {
            ""
        };

        // 6. 生成门牌号并组合地址
        let number = rand::random::<u16>() % 300 + 1;
        format!(
            "{}{}{}{}{}号",
            province,
            city,
            district,
            if street.is_empty() { "" } else { street },
            number
        )
    }

    pub fn chinese_postal_code(address: &str) -> String {
        // 省份前缀映射表（更完整的映射）
        let province_codes: HashMap<&str, &str> = [
            ("北京", "10"),
            ("上海", "20"),
            ("天津", "30"),
            ("重庆", "40"),
            ("河北", "05"),
            ("山西", "03"),
            ("辽宁", "11"),
            ("吉林", "13"),
            ("黑龙江", "15"),
            ("江苏", "21"),
            ("浙江", "31"),
            ("安徽", "23"),
            ("福建", "35"),
            ("江西", "33"),
            ("山东", "25"),
            ("河南", "45"),
            ("湖北", "43"),
            ("湖南", "41"),
            ("广东", "51"),
            ("广西", "53"),
            ("海南", "57"),
            ("四川", "61"),
            ("贵州", "55"),
            ("云南", "65"),
            ("西藏", "85"),
            ("陕西", "71"),
            ("甘肃", "73"),
            ("青海", "81"),
            ("宁夏", "75"),
            ("新疆", "83"),
            ("台湾", "88"),
            ("香港", "99"),
            ("澳门", "99"),
            ("内蒙古", "01"),
        ]
        .iter()
        .cloned()
        .collect();

        // 从地址中提取省份（匹配前2-4个字符）
        let province = province_codes
            .keys()
            .find(|&prov| address.starts_with(prov))
            .map(|s| *s)
            .unwrap_or_else(|| {
                // 如果没有匹配到省份，尝试匹配省级行政区全称
                let special_regions = ["自治区", "行政区", "省"];
                for region in &special_regions {
                    if let Some(pos) = address.find(region) {
                        let prov = &address[..pos + region.len()];
                        if province_codes.contains_key(prov) {
                            return prov;
                        }
                    }
                }
                // 默认返回随机省份代码
                "10" // 默认北京
            });

        let mut rng = rand::rng();

        // 获取省份代码前缀
        let prefix = province_codes
            .get(province)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:02}", rng.gen_range(10..99)));

        // 生成后4位（考虑特殊区域规则）
        let suffix = match province {
            "北京" | "上海" | "天津" | "重庆" => {
                // 直辖市市区使用000-099
                format!("{:02}0{:1}", rng.gen_range(0..10), rng.gen_range(0..10))
            }
            "香港" | "澳门" => "999".to_string(),
            _ => format!("{:04}", rng.gen_range(0..10000)),
        };

        format!("{}{}", prefix, suffix)
    }

    pub fn path_clean(path: &str) -> String {
        let path = path.trim_end_matches('/');
        // 使用'/'分割URL
        let parts: Vec<&str> = path.split('/').collect();
        // 初始化结果字符串
        let mut result = String::new();
        // 获取部分的总数
        let total_parts = parts.len();
        // 遍历每个部分及其索引
        for (index, part) in parts.iter().enumerate() {
            // 如果部分长度不超过255，则直接添加到结果字符串
            if part.len() <= 255 {
                result.push_str(part);
            } else {
                // 如果部分长度超过255，则截断并添加到结果字符串
                result.push_str(&part[..255]);
            }
            // 如果不是最后一段，则添加'/'分隔符
            if index < total_parts - 1 {
                result.push('/');
            } else if !part.contains(".") && !part.contains("?") {
                result.push_str(".html");
            }
        }
        if !result.contains("/") {
            format!("{}/index.html", result)
        } else {
            result
        }
    }

    pub fn encode_url_path(raw_url: &str) -> String {
        let mut url: String = raw_url.to_string();

        if raw_url.contains('%') {
            match decode(&raw_url) {
                Ok(decoded) => {
                    println!("url:{} Decoded: {}", url, decoded);
                    url = decoded.to_string();
                }
                Err(e) => println!("Error decoding: {}", e),
            }
        }

        // 手动定义不需要转义的安全字符
        let safe_chars = ['%', '?', ':', '/', '=', '&', '.', '-', '~', '#'];

        // 对路径部分进行编码
        let encoded_path: String = url
            .chars()
            .flat_map(|c| {
                if c.is_ascii_alphanumeric() || safe_chars.contains(&c) {
                    // 如果是字母、数字或安全字符，直接返回字符
                    vec![c.to_string()]
                } else {
                    // 否则，返回转义后的字符串（大写十六进制）
                    c.to_string()
                        .bytes()
                        .map(|b| format!("%{:02X}", b))
                        .collect()
                }
            })
            .collect();

        // println!("raw_url: {}\nencoded_path: {}", raw_url, encoded_path);
        encoded_path
    }

    // pub async fn get_new_link(
    //     &self,
    //     pgsql: &Arc<PgsqlService>,
    //     client: &Arc<Minio>,
    //     is_link_mapping: bool,
    //     link_strategy: &str,
    //     subdomain: &str,
    //     domain: &str,
    //     root_domain: &str,
    // ) -> String {
    //     if link_strategy.is_empty() {
    //         return "/".to_string();
    //     }

    //     // 分割并打乱策略顺序
    //     let mut strategies: Vec<&str> = link_strategy.split(',').collect();
    //     strategies.shuffle(&mut rand::rng());

    //     // 按随机顺序尝试各个策略
    //     for strategy in strategies {
    //         match strategy {
    //             // 1: 当前域名·内链
    //             "1" => {
    //                 let table_name = format!("{}__{}", subdomain, root_domain).replace(".", "_");
    //                 let page_type = if is_link_mapping { "映射" } else { "缓存" };
    //                 match get_cache_urls(pgsql, &table_name, page_type).await {
    //                     Some(urls) => {
    //                         if let Some(new_link) = urls.iter().choose(&mut rand::rng()) {
    //                             println!("从urls 抽中:{}", new_link);
    //                             return new_link.to_string();
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能从【{}】获取任何缓存url", table_name);
    //                     }
    //                 }
    //             }
    //             // 2: 主站·内链
    //             "2" => {
    //                 let full_domain = format!("www.{}", root_domain);
    //                 let config_path = format!("{root_domain}/{full_domain}.toml");
    //                 let page_type = match check_webconfig_is_mapping(&client, &config_path).await {
    //                     Some(is_mapping) => {
    //                         if is_mapping {
    //                             "映射"
    //                         } else {
    //                             "缓存"
    //                         }
    //                     }
    //                     None => {
    //                         println!("获取域名配置失败 或配置不存在");
    //                         continue;
    //                     }
    //                 };
    //                 let table_name = format!("www__{}", root_domain).replace(".", "_");
    //                 match get_cache_urls(pgsql, &table_name, page_type).await {
    //                     Some(urls) => {
    //                         if let Some(new_link) = urls.iter().choose(&mut rand::rng()) {
    //                             // println!("从urls 抽中:{}", new_link);
    //                             return new_link.to_string();
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能从【{}】获取任何缓存url", table_name);
    //                     }
    //                 }
    //             }
    //             // 3: 泛站·首页
    //             "3" => {
    //                 // 调用get_object_domains函数
    //                 match get_object_domains(client, "config", &format!("{root_domain}/")).await {
    //                     Some((www_domains, other_domains)) => {
    //                         // 成功获取域名列表
    //                         // println!("WWW域名:{:?}", www_domains);
    //                         // println!("其他域名:{:?}", other_domains);
    //                         if let Some(new_link) = other_domains.iter().choose(&mut rand::rng()) {
    //                             // println!("从other 抽中:{}", new_link);
    //                             return format!("//{}", new_link);
    //                         };
    //                     }
    //                     None => {
    //                         println!("未能获取任何域名");
    //                     }
    //                 }
    //             }
    //             // 4: 泛站·内链
    //             "4" => {
    //                 // 调用get_object_domains函数 随机抽一个泛站域名
    //                 let full_domain = match get_object_domains(
    //                     client,
    //                     "config",
    //                     &format!("{root_domain}/"),
    //                 )
    //                 .await
    //                 {
    //                     Some((www_domains, other_domains)) => {
    //                         // println!("WWW域名: {:?}", www_domains);
    //                         // println!("其他域名: {:?}", other_domains);

    //                         other_domains
    //                             .iter()
    //                             .choose(&mut rand::rng())
    //                             .map(|new_link| {
    //                                 // println!("从 other 抽中: {}", new_link);
    //                                 new_link.to_string()
    //                             })
    //                             .unwrap_or_else(|| {
    //                                 println!("other_domains 为空");
    //                                 String::new()
    //                             })
    //                     }
    //                     None => {
    //                         println!("未能获取任何域名");
    //                         String::new()
    //                     }
    //                 };

    //                 if full_domain.is_empty() {
    //                     continue;
    //                 }
    //                 // 获取站点配置文件中的映射开关状态
    //                 let config_path = format!("{root_domain}/{full_domain}.toml");
    //                 let page_type = match check_webconfig_is_mapping(&client, &config_path).await {
    //                     Some(is_mapping) => {
    //                         if is_mapping {
    //                             "映射"
    //                         } else {
    //                             "缓存"
    //                         }
    //                     }
    //                     None => {
    //                         println!("获取域名配置失败 或配置不存在");
    //                         continue;
    //                     }
    //                 };
    //                 let domain_info = domain_info_from_domain(&full_domain);
    //                 let table_name =
    //                     format!("{}__{}", domain_info["subdomain"], root_domain).replace(".", "_");
    //                 match get_cache_urls(pgsql, &table_name, page_type).await {
    //                     Some(urls) => {
    //                         if let Some(new_link) = urls.iter().choose(&mut rand::rng()) {
    //                             // println!("从urls 抽中:{}", new_link);
    //                             return new_link.to_string();
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能获取任何缓存url");
    //                     }
    //                 }
    //             }
    //             // 5: 【全服】主站·首页
    //             "5" => {
    //                 let table_name = "website_config";
    //                 let subdomain = "www";
    //                 match get_random_websites(pgsql, table_name, subdomain, "").await {
    //                     Some(websites) => {
    //                         // 从所有域名中随机选择一个
    //                         if let Some((domain, _)) =
    //                             websites.iter().choose(&mut rand::thread_rng())
    //                         {
    //                             println!("从urls抽中: {}", domain);
    //                             // 返回选中的域名
    //                             // 如果需要加上协议，可以这样：
    //                             return format!("https://{}", domain);
    //                         } else {
    //                             println!("字典中没有可用网站");
    //                             // return String::new(); // 或者返回默认值
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能从【{}】获取任何缓存url", table_name);
    //                         // return String::new(); // 或者返回默认值
    //                     }
    //                 }
    //             }
    //             // 6: 【全服】主站·内链
    //             "6" => {
    //                 // 调用get_object_domains函数 随机抽一个主站域名
    //                 let full_domain =
    //                     match get_object_domains(client, "config", &format!("/")).await {
    //                         Some((www_domains, other_domains)) => {
    //                             // println!("WWW域名: {:?}", www_domains);
    //                             // println!("其他域名: {:?}", other_domains);

    //                             www_domains
    //                                 .iter()
    //                                 .choose(&mut rand::rng())
    //                                 .map(|new_link| {
    //                                     // println!("从 www 抽中: {}", new_link);
    //                                     new_link.to_string()
    //                                 })
    //                                 .unwrap_or_else(|| {
    //                                     println!("www_domains 为空");
    //                                     String::new()
    //                                 })
    //                         }
    //                         None => {
    //                             println!("未能获取任何域名");
    //                             String::new()
    //                         }
    //                     };

    //                 if full_domain.is_empty() {
    //                     continue;
    //                 }
    //                 // 获取站点配置文件中的映射开关状态
    //                 let domain_info = domain_info_from_domain(&full_domain);
    //                 // let root_domain = &domain_info["root_domain"];
    //                 let config_path = format!(
    //                     "{}/{}.toml",
    //                     domain_info["root_domain"], domain_info["full_domain"]
    //                 );
    //                 let page_type = match check_webconfig_is_mapping(&client, &config_path).await {
    //                     Some(is_mapping) => {
    //                         if is_mapping {
    //                             "映射"
    //                         } else {
    //                             "缓存"
    //                         }
    //                     }
    //                     None => {
    //                         println!("获取域名配置失败 或配置不存在");
    //                         continue;
    //                     }
    //                 };
    //                 let table_name =
    //                     format!("www__{}", domain_info["root_domain"]).replace(".", "_");
    //                 match get_cache_urls(pgsql, &table_name, page_type).await {
    //                     Some(urls) => {
    //                         if let Some(new_link) = urls.iter().choose(&mut rand::rng()) {
    //                             // println!("从urls 抽中:{}", new_link);
    //                             return new_link.to_string();
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能获取任何缓存url");
    //                     }
    //                 }
    //             }
    //             // 7: 【全服】泛站·首页
    //             "7" => {
    //                 // 调用get_object_domains函数
    //                 match get_object_domains(client, "config", &format!("/")).await {
    //                     Some((www_domains, other_domains)) => {
    //                         // 成功获取域名列表
    //                         // println!("WWW域名:{:?}", www_domains);
    //                         // println!("其他域名:{:?}", other_domains);
    //                         if let Some(new_link) = other_domains.iter().choose(&mut rand::rng()) {
    //                             println!("从other 抽中:{}", new_link);
    //                             return format!("//{}", new_link);
    //                         };
    //                     }
    //                     None => {
    //                         println!("未能获取任何域名");
    //                     }
    //                 }
    //             }
    //             // 8: 【全服】泛站·内链
    //             "8" => {
    //                 // 调用get_object_domains函数 随机抽一个泛站域名
    //                 let full_domain =
    //                     match get_object_domains(client, "config", &format!("/")).await {
    //                         Some((www_domains, other_domains)) => {
    //                             // println!("WWW域名: {:?}", www_domains);
    //                             // println!("其他域名: {:?}", other_domains);
    //                             other_domains
    //                                 .iter()
    //                                 .choose(&mut rand::rng())
    //                                 .map(|new_link| {
    //                                     println!("从 others 抽中: {}", new_link);
    //                                     new_link.to_string()
    //                                 })
    //                                 .unwrap_or_else(|| {
    //                                     println!("other_domains 为空");
    //                                     String::new()
    //                                 })
    //                         }
    //                         None => {
    //                             println!("未能获取任何域名");
    //                             String::new()
    //                         }
    //                     };
    //                 if full_domain.is_empty() {
    //                     continue;
    //                 }
    //                 // 获取站点配置文件中的映射开关状态
    //                 let domain_info = domain_info_from_domain(&full_domain);
    //                 let config_path = format!(
    //                     "{}/{}.toml",
    //                     domain_info["root_domain"], domain_info["full_domain"]
    //                 );
    //                 let page_type = match check_webconfig_is_mapping(&client, &config_path).await {
    //                     Some(is_mapping) => {
    //                         if is_mapping {
    //                             "映射"
    //                         } else {
    //                             "缓存"
    //                         }
    //                     }
    //                     None => {
    //                         println!("获取域名配置失败 或配置不存在");
    //                         continue;
    //                     }
    //                 };
    //                 let table_name = format!(
    //                     "{}__{}",
    //                     domain_info["subdomain"], domain_info["root_domain"]
    //                 )
    //                 .replace(".", "_");
    //                 match get_cache_urls(pgsql, &table_name, page_type).await {
    //                     Some(urls) => {
    //                         if let Some(new_link) = urls.iter().choose(&mut rand::rng()) {
    //                             // println!("从urls 抽中:{}", new_link);
    //                             return new_link.to_string();
    //                         }
    //                     }
    //                     None => {
    //                         println!("未能获取任何缓存url");
    //                     }
    //                 }
    //             }
    //             // 默认回退
    //             _ => continue,
    //         }
    //     }

    //     // 所有策略都失败时返回默认值
    //     "/".to_string()
    // }
    // pub fn encode_url_path(raw_url: &str) -> String {
    //     // 手动定义不需要转义的安全字符
    //     let safe_chars = ['%', '?', ':', '/', '=', '&', '.', '-', '~', '#'];

    //     // 对路径部分进行编码
    //     let encoded_path: String = raw_url
    //         .chars()
    //         .flat_map(|c| {
    //             if c.is_ascii_alphanumeric() || safe_chars.contains(&c) {
    //                 // 如果是字母、数字或安全字符，直接返回字符
    //                 vec![c.to_string()]
    //             } else {
    //                 // 否则，返回转义后的字符串（大写十六进制）
    //                 c.to_string()
    //                     .bytes()
    //                     .map(|b| format!("%{:02X}", b))
    //                     .collect()
    //             }
    //         })
    //         .collect();

    //     // println!("raw_url: {}\nencoded_path: {}", raw_url, encoded_path);
    //     encoded_path
    // }

    pub async fn get_push_link(
        &self,
        linecache: &Arc<AsyncLineCache>,
        path: String,
    ) -> (String, String) {
        let mut tag_cache: HashMap<String, String> = HashMap::new(); // 局部缓存
        let mut title_parts = Vec::new();

        // 1. 获取随机行
        let mut push_link = match linecache.get_lines(&path).await {
            Ok(Some(lines)) if !lines.is_empty() => {
                lines.choose(&mut rand::rng()).cloned().unwrap_or_default()
            }
            Ok(Some(_)) => {
                println!("文件为空");
                "".to_string()
            }
            Ok(None) => {
                println!("文件未找到");
                "".to_string()
            }
            Err(e) => {
                eprintln!("无法打开日志文件: {}", e);
                "".to_string()
            }
        };

        // 2. 替换 {tag} 占位符（带缓存）
        let mut replacements = Vec::new();
        if !push_link.is_empty() {
            for cap in KUO_HAO_REGEX.captures_iter(&push_link) {
                let tag_name = &cap[1];

                // 优先使用缓存值
                let tag_value = if let Some(cached) = tag_cache.get(tag_name) {
                    cached.clone()
                } else {
                    // 无缓存则获取新值
                    let tag_path = format!("doc/{}.txt", tag_name);
                    let new_value = if tag_name == " " {
                        tag_name.to_string() // 如果是空格标签，直接返回空格
                    } else {
                        match linecache.get_lines(&tag_path).await {
                            Ok(Some(lines)) if !lines.is_empty() => {
                                lines.choose(&mut rand::rng()).cloned().unwrap_or_default()
                            }
                            _ => {
                                println!("无法获取 tag: {}", tag_name);
                                "".to_string()
                            }
                        }
                    };
                    tag_cache.insert(tag_name.to_string(), new_value.clone()); // 存入缓存
                    new_value
                };

                title_parts.push(tag_value.clone()); // 收集用于title
                replacements.push((tag_name.to_string(), tag_value));
                // push_link = push_link.replace(&format!("{{{}}}", tag_name), &tag_value);
            }
            // 统一执行替换
            for (tag_name, tag_value) in replacements {
                push_link = push_link.replace(&format!("{{{}}}", tag_name), &tag_value);
            }
        }

        // 3. 替换 【uuid】
        if push_link.contains("【uuid】") {
            let uuid = Uuid::new_v4().to_string().replace("-", "").to_uppercase();
            push_link = push_link.replace("【uuid】", &uuid);
        }

        // 4. 生成title
        let title = title_parts.join("");

        (push_link, title)
    }

    pub async fn get_log_datas(
        date: &str,
        qps: bool,
        linecache: &Arc<AsyncLineCache>,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        // 解析精确日期
        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| format!("无效的日期格式 '{}': {}", date, e))?;
        let date_str = target_date.format("%Y-%m-%d").to_string();
        let file_path = format!("log/app.log.{}", date_str);

        // 存储解析后的日志数据
        let mut log_entries = Vec::new();

        if qps {
            // 异步打开文件；如果文件不存在，返回空数组
            let mut file = match File::open(&file_path).await {
                Ok(file) => file,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(json!([]));
                }
                Err(err) => {
                    // eprintln!("无法打开日志文件 {}: {}", file_path, err);
                    return Err(Box::new(err));
                }
            };
            // QPS 模式：从文件末尾读取最近 5 秒的数据
            let target_time = Local::now() - Duration::seconds(5);
            let mut buffer = Vec::new();
            let mut chunk = vec![0u8; 4096]; // 每次读取 4KB
            let mut pos = file.seek(SeekFrom::End(0)).await? as i64;
            let mut stop = false;

            while pos > 0 && !stop {
                // 计算读取大小
                let read_size = if pos >= chunk.len() as i64 {
                    chunk.len()
                } else {
                    pos as usize
                };
                pos -= read_size as i64;
                file.seek(SeekFrom::Start(pos as u64)).await?;
                let n = file.read(&mut chunk[..read_size]).await?;
                if n == 0 {
                    eprintln!("没有更多数据可读取");
                    break;
                }
                buffer.splice(0..0, chunk[..n].iter().cloned());

                // 将 buffer 解码为 UTF-8 字符串
                let s = match std::str::from_utf8(&buffer) {
                    Ok(s) => s.to_string(),
                    Err(e) => {
                        // 处理可能的非 UTF-8 边界
                        let valid_len = e.valid_up_to();
                        if valid_len == 0 {
                            eprintln!("无效的 UTF-8 数据，跳过块");
                            buffer.clear();
                            continue;
                        }
                        let valid_str = std::str::from_utf8(&buffer[..valid_len])?.to_string();
                        buffer.drain(..valid_len);
                        valid_str
                    }
                };

                // 从字符串中提取行（从后向前）
                let mut lines = Vec::new();
                let mut line = String::new();
                for c in s.chars().rev() {
                    if c == '\n' && !line.is_empty() {
                        lines.push(line.chars().rev().collect::<String>());
                        line.clear();
                    } else {
                        line.push(c);
                    }
                }
                if !line.is_empty() {
                    lines.push(line.chars().rev().collect::<String>());
                }

                // 处理行（从新到旧）
                for line in lines.iter() {
                    if line.is_empty() {
                        // eprintln!("跳过空行");
                        continue;
                    }
                    let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                    if parts.len() != 6 {
                        // eprintln!("无效的日志格式 ({} 部分): {}", parts.len(), line);
                        continue;
                    }
                    match DateTime::parse_from_rfc3339(parts[0]) {
                        Ok(timestamp) => {
                            let timestamp = timestamp.with_timezone(&Local);
                            if timestamp < target_time {
                                // eprintln!(
                                //     "停止读取: 时间戳 {} <= 目标时间 {}",
                                //     timestamp, target_time
                                // );
                                stop = true;
                                break;
                            }
                            // eprintln!("有效记录: 时间戳 {}", timestamp);
                            let log_entry = json!({
                                "timestamp": parts[0],
                                "user_type": parts[1],
                                "ip": parts[2],
                                "request_url": parts[3],
                                "referrer": parts[4],
                                "user_agent": parts[5]
                            });
                            log_entries.push(log_entry);
                        }
                        Err(e) => {
                            eprintln!("无效的时间戳格式 '{}': {}", parts[0], e);
                            continue;
                        }
                    }
                }

                // 保留未完成的行（字节形式）
                if pos > 0 && !stop && !lines.is_empty() {
                    buffer = lines[0].chars().rev().collect::<String>().into_bytes();
                } else {
                    buffer.clear();
                }
            }
        } else {
            // 正常模式：从头读取整个文件
            // let reader = BufReader::new(file);
            // let mut lines = reader.lines();
            match linecache.get_lines(&file_path).await {
                Ok(Some(lines)) => {
                    // println!("bind_domains:{:?}", bind_domains);
                    for line in lines {
                        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                        if parts.len() == 6 {
                            let log_entry = json!({
                                "timestamp": parts[0],
                                "user_type": parts[1],
                                "ip": parts[2],
                                "request_url": parts[3],
                                "referrer": parts[4],
                                "user_agent": parts[5]
                            });
                            log_entries.push(log_entry);
                        }
                        // else {
                        //     eprintln!("无效的日志格式: {}", line);
                        // }
                    }
                }
                Ok(None) => {
                    println!("文件未找到或为空");
                }
                Err(e) => {
                    // eprintln!("无法打开日志文件: {}", e);
                }
            }
        }

        // 返回 JSON 格式的日志数据

        Ok(json!(log_entries))
    }

    // pub async fn get_log_datas(day: u32) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    //     // 计算目标日期
    //     let target_date = Local::now() - Duration::days(day as i64);
    //     let date_str = target_date.format("%Y-%m-%d").to_string();
    //     let file_path = format!("log/app.log.{}", date_str);

    //     // 异步打开文件，如果文件不存在则返回空数组
    //     let file = match File::open(&file_path).await {
    //         Ok(file) => file,
    //         Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
    //             return Ok(json!([])); // 文件不存在时返回空数组
    //         }
    //         Err(err) => {
    //             eprintln!("Failed to open log file: {}", err);
    //             return Err(Box::new(err));
    //         }
    //     };

    //     // 使用 BufReader 逐行读取文件
    //     let reader = BufReader::new(file);
    //     let mut lines = reader.lines();

    //     // 存储解析后的日志数据
    //     let mut log_entries = Vec::new();

    //     // 逐行处理日志文件
    //     while let Some(line) = lines.next_line().await? {
    //         // 按 | 分割每一行
    //         let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

    //         // 确保每行有 6 个部分
    //         if parts.len() == 6 {
    //             let log_entry = json!({
    //                 "timestamp": parts[0],
    //                 "user_type": parts[1],
    //                 "ip": parts[2],
    //                 "request_url": parts[3],
    //                 "referrer": parts[4],
    //                 "user_agent": parts[5]
    //             });
    //             log_entries.push(log_entry);
    //         }
    //         // else {
    //         //     eprintln!("Invalid log format: {}", line);
    //         // }
    //     }

    //     // 返回 JSON 数据
    //     Ok(json!(log_entries))
    // }

    // pub async fn minio_delete_empty_prefix(
    //     minio_client: Arc<MinioClient>,
    //     client: Arc<Minio>,
    //     bucket: &str,
    //     prefix: &str,
    // ) -> Result<bool, Box<dyn Error>> {
    //     let first_part = prefix
    //         .trim_matches('/') // 去掉首尾的/
    //         .split('/')
    //         .next()
    //         .unwrap_or(""); // 获取第一个部分
    //     if first_part.len()<1{
    //         println!("前缀小于1，禁止删除");
    //         return Ok(false);
    //     }
    //     let normalized_prefix = format!("{}", first_part);
    //     let objects_args = ListObjectsArgs::default()
    //         .max_keys(1)
    //         .prefix(&normalized_prefix); // 注意：不要用 delimiter，否则只会返回下一级

    //     let list_result = client.list_objects(bucket, objects_args).await?;

    //     // 3. 如果存在对象或子前缀，则非空
    //     if !list_result.contents.is_empty() || !list_result.common_prefixes.is_empty(){
    //         println!(
    //             "Prefix '{}' is NOT empty (objects: {}, sub-prefixes: {})",
    //             normalized_prefix,
    //             list_result.contents.len(),
    //             list_result.common_prefixes.len()
    //         );
    //         return Ok(false);
    //     }
    //     // 删除所有版本
    //     // println!("删除 normalized_prefix: {}",normalized_prefix);
    //     minio_client
    //         .delete_all_versions(bucket, &normalized_prefix)
    //         .await?;
    //     Ok(true)
    // }
    pub fn extract_metadata(page_source: &str) -> MetaData {
        let mut metadata = MetaData::default();

        // 提取标题
        let title_re = Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap();
        if let Some(caps) = title_re.captures(page_source) {
            if let Some(title_match) = caps.get(1) {
                let title = title_match
                    .as_str()
                    .trim()
                    .replace(['\n', '\r'], " ")
                    .replace("&#32;", " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                metadata.title = Some(title);
            }
        }

        // 提取 keywords 和 description
        let meta_re = Regex::new(
            r#"(?is)<meta\s+[^>]*name\s*=\s*["']([^"']+)["']\s+[^>]*content\s*=\s*["']([^"']+)["'][^>]*>"#,
        ).unwrap();

        for caps in meta_re.captures_iter(page_source) {
            if let Some(name_match) = caps.get(1) {
                let name = name_match.as_str().to_lowercase();
                if let Some(content_match) = caps.get(2) {
                    let content = content_match.as_str().to_string();

                    match name.as_str() {
                        "keywords" => metadata.keywords = Some(content),
                        "description" => metadata.description = Some(content),
                        _ => (),
                    }
                }
            }
        }

        metadata
    }

    pub async fn get_country_city(&self, ip: &str) -> String {
        // 将字符串 IP 地址转换为 Ipv4Addr
        match ip.parse::<Ipv4Addr>() {
            Ok(ip_addr) => {
                // 查询 IP 地址
                match self.ipdb.lookup_ipv4(ip_addr, None).await {
                    Ok(record_option) => {
                        if let Some(record) = record_option {
                            // 构造国家和城市字符串
                            let country = record.country_name.unwrap_or_default();
                            let city = record.city_name.unwrap_or_default();
                            if country == city {
                                format!(" {}", country)
                            } else {
                                format!(" {} - {}", country, city)
                            }
                        } else {
                            // 如果查询失败或记录为空，返回空字符串
                            "".to_string()
                        }
                    }
                    Err(_) => {
                        // 如果查询失败，返回空字符串
                        "".to_string()
                    }
                }
            }
            Err(_) => {
                // 如果 IP 地址格式不正确，返回空字符串
                "".to_string()
            }
        }
    }

    pub fn encode_to_html_entities(text: &str) -> String {
        text.chars()
            .map(|c| {
                if c == ',' {
                    // 直接保留逗号
                    c.to_string()
                } else if c.is_ascii() {
                    format!("&#{};", c as u32)
                } else {
                    format!("&#x{:X};", c as u32)
                }
            })
            .collect()
    }

    pub async fn tag_parse(
        &self,
        config_dict: &Config,
        webconfig: &WebsiteConf,
        req_state: &RequestState,
        pgsql: &Arc<PgsqlService>,
        linecache: &Arc<AsyncLineCache>,
        source: String,
    ) -> String {
        use std::time::Instant;
        const MAX_ITERATIONS: usize = 100;
        let mut result = source;
        let mut iteration_count = 0;
        // let mut webconfig_dict: HashMap<String, WebsiteConf> = HashMap::new();
        // let mut id_domain_dict: HashMap<String, String> = HashMap::new();
        let mut webconfig_dict: DashMap<String, WebsiteConf> = DashMap::new();
        let mut id_domain_dict: DashMap<String, String> = DashMap::new();

        #[derive(Debug)]
        struct TagMatch {
            full_match: String,
            start_pos: usize,
            end_pos: usize,
            replacement: String,
            depth: usize,
            id_str: String,
            tag_type: &'static str,
        }

        while iteration_count < MAX_ITERATIONS {
            iteration_count += 1;
            let mut changed = false;
            let mut output = result.clone();
            let mut tag_stack: Vec<TagMatch> = Vec::new();

            // 1. 扫描所有标签
            let mut current_pos = 0;
            let mut depth = 0;
            let tag_patterns = [
                (&FUNC_TAG_REGEX, "func"),
                (&DOC_TAG_REGEX, "doc"),
                (&FIXED_TAG_REGEX, "fixed"),
            ];

            while current_pos < output.len() {
                let mut earliest_match: Option<(usize, usize)> = None;
                let mut earliest_tag_type = "";
                let mut earliest_cap = None;

                for (regex, tag_type) in tag_patterns.iter() {
                    if let Some(cap) = regex.captures(&output[current_pos..]) {
                        let m = cap.get(0).unwrap();
                        let start = current_pos + m.start();
                        let end = current_pos + m.end();
                        if end <= output.len()
                            && output.is_char_boundary(start)
                            && output.is_char_boundary(end)
                        {
                            if earliest_match.is_none() || start < earliest_match.unwrap().0 {
                                earliest_match = Some((start, end));
                                earliest_tag_type = tag_type;
                                earliest_cap = Some(cap);
                            }
                        } else {
                            eprintln!(
                                "警告: 无效匹配范围: regex={}, start={}, end={}, len={}",
                                regex.as_str(),
                                start,
                                end,
                                output.len()
                            );
                            current_pos += 1;
                            continue;
                        }
                    }
                }

                if let Some((start, end)) = earliest_match {
                    let full_match = output[start..end].to_string();
                    if full_match.contains('{')
                        && !full_match.starts_with("{!")
                        && !full_match.starts_with("{@")
                        && !full_match.starts_with("{*@")
                    {
                        depth += 1;
                    }
                    let id_str = earliest_cap
                        .as_ref()
                        .and_then(|cap| cap.get(cap.len() - 1).map(|m| m.as_str().to_string()))
                        .unwrap_or_default();
                    tag_stack.push(TagMatch {
                        full_match,
                        start_pos: start,
                        end_pos: end,
                        replacement: String::new(),
                        depth,
                        id_str,
                        tag_type: earliest_tag_type,
                    });
                    current_pos = end;
                } else {
                    break;
                }
            }

            // 2. 从最内层处理标签
            tag_stack.sort_by_key(|a| std::cmp::Reverse(a.depth));

            for tag_match in tag_stack.iter_mut() {
                let start_time = Instant::now();
                let s = tag_match.full_match.as_str();
                if s.starts_with("{ ") || s.ends_with(" }") {
                    continue;
                }
                // eprintln!(
                // "DEBUG: 处理标签: {} (type: {}, depth: {}, id_str: {}, pos: {}-{})",
                //     s,
                //     tag_match.tag_type,
                //     tag_match.depth,
                //     tag_match.id_str,
                //     tag_match.start_pos,
                //     tag_match.end_pos
                // );

                match tag_match.tag_type {
                    "func" => {
                        if let Some(cap) = FUNC_TAG_REGEX.captures(s) {
                            let func_name = cap.get(1).unwrap().as_str();
                            let params_str = cap.get(2).unwrap().as_str();
                            tag_match.id_str = cap.get(3).map_or("", |m| m.as_str()).to_string();
                            // eprintln!(
                            // "DEBUG: 函数标签: name={}, params={}, id_str={}",
                            // func_name, params_str, tag_match.id_str
                            // );

                            tag_match.replacement = match func_name {
                                "GetFileName" => params_str
                                    .split(',')
                                    .next()
                                    .map(|path| {
                                        std::path::Path::new(path.trim())
                                            .file_name()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("invalid_path")
                                            .to_string()
                                    })
                                    .unwrap_or_else(|| "default.txt".into()),
                                "ToUpper" => params_str
                                    .split(',')
                                    .next()
                                    .map(|s| s.trim().to_uppercase())
                                    .unwrap_or_default(),
                                "Substring" => {
                                    // 分割参数
                                    let params: Vec<&str> =
                                        params_str.split(',').map(|s| s.trim()).collect();
                                    if params.len() < 3 {
                                        "参数不足".into()
                                    } else {
                                        let s = params[0];
                                        let start = params[1].parse().unwrap_or(0);
                                        let len = params[2].parse().unwrap_or(s.len());
                                        s.chars().skip(start).take(len).collect()
                                    }
                                }
                                "Add" => {
                                    let mut result = 0.0;
                                    for param in params_str.split(',') {
                                        if let Ok(num) = param.trim().parse::<f64>() {
                                            result += num;
                                        }
                                    }
                                    result.to_string()
                                }
                                _ => {
                                    eprintln!("警告: 未定义的函数: {}", func_name);
                                    String::new()
                                }
                            };
                        }
                    }
                    "doc" => {
                        if let Some(cap) = DOC_TAG_REGEX.captures(s) {
                            let prefix = cap.get(1).unwrap().as_str();
                            let is_doc2 = prefix == "^@";
                            let is_doc3 = prefix == ":@";
                            let doc_name = cap.get(2).unwrap().as_str();
                            let params_str = cap.get(3).map_or("", |m| m.as_str()).to_string();
                            tag_match.id_str = cap.get(4).map_or("", |m| m.as_str()).to_string();
                            // eprintln!(
                            // "DEBUG: 文档标签: prefix={}, name={}, params={}, id_str={}",
                            // prefix, doc_name, params_str, tag_match.id_str
                            // );
                            let params: Vec<String> = self
                                .parse_params(&params_str)
                                .into_iter()
                                .map(|s| {
                                    // Remove outer single quotes (if fully wrapped)
                                    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
                                        s[1..s.len() - 1].to_string()
                                    } else {
                                        s
                                    }
                                })
                                .collect();

                            // println!("params: {:?}", params);
                            let mut count = 1;
                            let mut join_str = String::new();

                            for (index, param) in params.iter().enumerate() {
                                match index {
                                    0 => {
                                        // 处理范围格式如 "2_8" 或单个数字
                                        if param.contains('_') {
                                            let parts: Vec<&str> = param.split('_').collect();
                                            if parts.len() == 2 {
                                                if let (Ok(min), Ok(max)) = (
                                                    parts[0].parse::<usize>(),
                                                    parts[1].parse::<usize>(),
                                                ) {
                                                    count = rand::rng().gen_range(min..=max);
                                                }
                                            }
                                        } else if let Ok(num) = param.parse::<usize>() {
                                            count = num;
                                        }
                                    }
                                    1 => join_str = param.clone(),
                                    _ => eprintln!("警告: 忽略参数: {}", param),
                                }
                            }
                            // 处理文档路径
                            if is_doc3 {
                                let mut file_dir = Path::new("doc").join(format!("{}", doc_name));
                                if !file_dir.exists() {
                                    file_dir = Path::new(&doc_name).to_path_buf();
                                }
                                tag_match.replacement = self
                                    .get_random_filename(&file_dir, None, usize::MAX)
                                    .unwrap_or_else(|| "未找到文件".to_string());
                            } else {
                                let mut replacement = String::new();
                                match self.get_doc_path(doc_name) {
                                    Some(file_path) => {
                                        for i in 0..count {
                                            let line = if is_doc2 {
                                                linecache.random_sign(&file_path).await
                                            } else {
                                                linecache.random_line(&file_path).await
                                            };
                                            match line {
                                                Ok(Some(line)) => {
                                                    let text = if join_str.contains("...") {
                                                        join_str.replace("...", &line)
                                                    } else {
                                                        if i + 1 == count {
                                                            line
                                                        } else {
                                                            format!("{}{}", line, join_str)
                                                        }
                                                    };
                                                    replacement.push_str(&text);
                                                }
                                                Ok(None) => {
                                                    eprintln!("警告: 文档为空: {}", file_path)
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "错误: 无法读取文档 {}: {}",
                                                        file_path, e
                                                    )
                                                }
                                            }
                                        }
                                        tag_match.replacement = replacement;
                                    }
                                    None => println!("{} 目录中没有符合条件的文件", doc_name), // 明确处理空结果
                                }
                            }
                        }
                    }
                    "fixed" => {
                        if let Some(cap) = FIXED_TAG_REGEX.captures(s) {
                            let tag_name = cap.get(1).unwrap().as_str();
                            tag_match.id_str = cap.get(2).map_or("", |m| m.as_str()).to_string();
                            // eprintln!(
                            // "DEBUG: 固定标签: name={}, id_str={}",
                            // tag_name, tag_match.id_str
                            // );

                            tag_match.replacement = match tag_name {
                                "根域名" | "root_domain" | "*根域名" | "*root_domain" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "www",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                domain_info["root_domain"].to_string()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        req_state
                                            .domain_info
                                            .get("root_domain")
                                            .cloned()
                                            .unwrap_or_default()
                                    }
                                }
                                "域名" | "domain" | "*域名" | "*domain" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                domain_info["full_domain"].to_string()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        req_state
                                            .domain_info
                                            .get("full_domain")
                                            .cloned()
                                            .unwrap_or_default()
                                    }
                                }
                                "网址" | "url" | "*网址" | "*url" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                let domain = domain_info["full_domain"].to_string();
                                                self.get_domain_url(pgsql, &domain).await
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        req_state.url.clone()
                                    }
                                }
                                "随机网址" | "rand_url" => {
                                    let domain = req_state.domain_info["full_domain"].to_string();
                                    self.get_domain_url(pgsql, &domain).await
                                }
                                "首页" | "index" | "*首页" | "*index" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                let domain = domain_info["full_domain"].to_string();
                                                format!("http://{}", domain)
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        let domain =
                                            req_state.domain_info["full_domain"].to_string();
                                        format!("http://{}", domain)
                                    }
                                }
                                "标题" | "title" | "*标题" | "*title" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                // let domain = domain_info["full_domain"].to_string();
                                                // format!("http://{}", domain)

                                                let domain = domain_info["full_domain"].to_string();
                                                let config_path = format!(
                                                    "{}/{}.toml",
                                                    domain_info["root_domain"], domain
                                                );

                                                let new_webconfig = if let Some(webconfig) =
                                                    webconfig_dict.get(&domain)
                                                {
                                                    // println!("{} 配置缓存存在，直接返回", domain);
                                                    webconfig.clone()
                                                } else {
                                                    // println!("Domain {} not found, fetching...", domain);
                                                    match self
                                                        .fetch_or_create_config(
                                                            true,
                                                            config_dict,
                                                            &pgsql,
                                                            &config_path,
                                                            &domain,
                                                        )
                                                        .await
                                                    {
                                                        Ok(webconfig) => {
                                                            webconfig_dict.insert(
                                                                domain.clone(),
                                                                webconfig.clone(),
                                                            );
                                                            webconfig
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to fetch config: {}",
                                                                e
                                                            );
                                                            return format!("{}?", tag_name);
                                                        }
                                                    }
                                                };
                                                new_webconfig.info.title.clone()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        webconfig.info.title.clone()
                                    }
                                }
                                "关键词组" | "keywords" | "*关键词组" | "*keywords" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                let domain = domain_info["full_domain"].to_string();
                                                let config_path = format!(
                                                    "{}/{}.toml",
                                                    domain_info["root_domain"], domain
                                                );

                                                let new_webconfig = if let Some(webconfig) =
                                                    webconfig_dict.get(&domain)
                                                {
                                                    // println!("{} 配置缓存存在，直接返回", domain);
                                                    webconfig.clone()
                                                } else {
                                                    // println!("Domain {} not found, fetching...", domain);
                                                    match self
                                                        .fetch_or_create_config(
                                                            true,
                                                            config_dict,
                                                            &pgsql,
                                                            &config_path,
                                                            &domain,
                                                        )
                                                        .await
                                                    {
                                                        Ok(webconfig) => {
                                                            webconfig_dict.insert(
                                                                domain.clone(),
                                                                webconfig.clone(),
                                                            );
                                                            webconfig
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to fetch config: {}",
                                                                e
                                                            );
                                                            return format!("{}?", tag_name);
                                                        }
                                                    }
                                                };
                                                new_webconfig.info.keywords.clone()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        webconfig.info.keywords.clone()
                                    }
                                }
                                "关键词" | "keyword" | "*关键词" | "*keyword" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                // let domain = domain_info["full_domain"].to_string();
                                                // format!("http://{}", domain)

                                                let domain = domain_info["full_domain"].to_string();
                                                let config_path = format!(
                                                    "{}/{}.toml",
                                                    domain_info["root_domain"], domain
                                                );

                                                let new_webconfig = if let Some(webconfig) =
                                                    webconfig_dict.get(&domain)
                                                {
                                                    // println!("{} 配置缓存存在，直接返回", domain);
                                                    webconfig.clone()
                                                } else {
                                                    // println!("Domain {} not found, fetching...", domain);
                                                    match self
                                                        .fetch_or_create_config(
                                                            true,
                                                            config_dict,
                                                            &pgsql,
                                                            &config_path,
                                                            &domain,
                                                        )
                                                        .await
                                                    {
                                                        Ok(webconfig) => {
                                                            webconfig_dict.insert(
                                                                domain.clone(),
                                                                webconfig.clone(),
                                                            );
                                                            webconfig
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to fetch config: {}",
                                                                e
                                                            );
                                                            return format!("{}?", tag_name);
                                                        }
                                                    }
                                                };
                                                let keywords: Vec<&str> = new_webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .map(|s| s.trim())
                                                    .filter(|s| !s.is_empty())
                                                    .collect();
                                                keywords
                                                    .choose(&mut rand::rng())
                                                    .map(|s| s.to_string())
                                                    .unwrap_or("".to_string())
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        let keywords: Vec<&str> = webconfig
                                            .info
                                            .keywords
                                            .split(',')
                                            .map(|s| s.trim())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        keywords
                                            .choose(&mut rand::rng())
                                            .map(|s| s.to_string())
                                            .unwrap_or("".to_string())
                                    }
                                }
                                "核心词" | "coreword" | "*核心词" | "*coreword" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                // let domain = domain_info["full_domain"].to_string();
                                                // format!("http://{}", domain)

                                                let domain = domain_info["full_domain"].to_string();
                                                let config_path = format!(
                                                    "{}/{}.toml",
                                                    domain_info["root_domain"], domain
                                                );

                                                let new_webconfig = if let Some(webconfig) =
                                                    webconfig_dict.get(&domain)
                                                {
                                                    // println!("{} 配置缓存存在，直接返回", domain);
                                                    webconfig.clone()
                                                } else {
                                                    println!(
                                                        "Domain {} not found, fetching...",
                                                        domain
                                                    );
                                                    match self
                                                        .fetch_or_create_config(
                                                            true,
                                                            config_dict,
                                                            &pgsql,
                                                            &config_path,
                                                            &domain,
                                                        )
                                                        .await
                                                    {
                                                        Ok(webconfig) => {
                                                            webconfig_dict.insert(
                                                                domain.clone(),
                                                                webconfig.clone(),
                                                            );
                                                            webconfig
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to fetch config: {}",
                                                                e
                                                            );
                                                            return format!("{}?", tag_name);
                                                        }
                                                    }
                                                };
                                                new_webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .find_map(|s| {
                                                        let s = s.trim();
                                                        (!s.is_empty()).then(|| s.to_owned())
                                                    })
                                                    .unwrap_or_default()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        webconfig
                                            .info
                                            .keywords
                                            .split(',')
                                            .find_map(|s| {
                                                let s = s.trim();
                                                (!s.is_empty()).then(|| s.to_owned())
                                            })
                                            .unwrap_or_default()
                                    }
                                }
                                "描述" | "description" | "*描述" | "*description" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                // let domain = domain_info["full_domain"].to_string();
                                                // format!("http://{}", domain)

                                                let domain = domain_info["full_domain"].to_string();
                                                let config_path = format!(
                                                    "{}/{}.toml",
                                                    domain_info["root_domain"], domain
                                                );

                                                let new_webconfig = if let Some(webconfig) =
                                                    webconfig_dict.get(&domain)
                                                {
                                                    // println!("{} 配置缓存存在，直接返回", domain);
                                                    webconfig.clone()
                                                } else {
                                                    // println!("Domain {} not found, fetching...", domain);
                                                    match self
                                                        .fetch_or_create_config(
                                                            true,
                                                            config_dict,
                                                            &pgsql,
                                                            &config_path,
                                                            &domain,
                                                        )
                                                        .await
                                                    {
                                                        Ok(webconfig) => {
                                                            webconfig_dict.insert(
                                                                domain.clone(),
                                                                webconfig.clone(),
                                                            );
                                                            webconfig
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to fetch config: {}",
                                                                e
                                                            );
                                                            return format!("{}?", tag_name);
                                                        }
                                                    }
                                                };
                                                new_webconfig.info.description.clone()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        webconfig.info.description.clone()
                                    }
                                }
                                "主站.域名" | "main.domain" | "*主站.域名" | "*main.domain" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "www",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                domain_info["full_domain"].to_string()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        if req_state.domain_info["subdomain"] == "www"
                                            || req_state.domain_info["subdomain"] == ""
                                        {
                                            "{域名}".to_string()
                                        } else {
                                            format!("www.{}", req_state.domain_info["root_domain"])
                                        }
                                    }
                                }
                                "主站.网址" | "main.url" | "*主站.网址" | "*main.url" => {
                                    let domain = if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "www",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                domain_info["full_domain"].to_string()
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        format!("www.{}", req_state.domain_info["root_domain"])
                                    };
                                    self.get_domain_url(pgsql, &domain).await
                                }
                                "主站.首页" | "main.index" | "*主站.首页" | "*main.index" => {
                                    if tag_name.starts_with("*") {
                                        match self
                                            .random_domain_info(
                                                pgsql,
                                                tag_match.id_str.to_string(),
                                                &mut id_domain_dict,
                                                "www",
                                                "",
                                            )
                                            .await
                                        {
                                            Some(domain_info) => {
                                                format!(
                                                    "http://{}",
                                                    domain_info["full_domain"].to_string()
                                                )
                                            }
                                            None => String::new(),
                                        }
                                    } else {
                                        if req_state.domain_info["subdomain"] == "www"
                                            || req_state.domain_info["subdomain"] == ""
                                        {
                                            "{首页}".to_string()
                                        } else {
                                            format!(
                                                "http://www.{}",
                                                req_state.domain_info["root_domain"]
                                            )
                                        }
                                    }
                                }
                                "主站.标题"
                                | "主站.关键词组"
                                | "主站.关键词"
                                | "主站.核心词"
                                | "主站.描述"
                                | "main.title"
                                | "main.keywords"
                                | "main.keyword"
                                | "main.coreword"
                                | "main.description"
                                | "*主站.标题"
                                | "*主站.关键词组"
                                | "*主站.关键词"
                                | "*主站.核心词"
                                | "*主站.描述"
                                | "*main.title"
                                | "*main.keywords"
                                | "*main.keyword"
                                | "*main.coreword"
                                | "*main.description" => {
                                    if (req_state.domain_info["subdomain"] == "www"
                                        || req_state.domain_info["subdomain"] == "")
                                        && !tag_name.starts_with("*")
                                    {
                                        if tag_name.ends_with("标题") || tag_name.ends_with("title")
                                        {
                                            webconfig.info.title.clone()
                                        } else if tag_name.ends_with("关键词组")
                                            || tag_name.ends_with("keywords")
                                        {
                                            webconfig.info.keywords.clone()
                                        } else if tag_name.ends_with("关键词")
                                            || tag_name.ends_with("keyword")
                                        {
                                            let keywords: Vec<&str> = webconfig
                                                .info
                                                .keywords
                                                .split(',')
                                                .map(|s| s.trim())
                                                .filter(|s| !s.is_empty())
                                                .collect();
                                            keywords
                                                .choose(&mut rand::rng())
                                                .map(|s| s.to_string())
                                                .unwrap_or("".to_string())
                                        } else if tag_name.ends_with("核心词")
                                            || tag_name.ends_with("coreword")
                                        {
                                            webconfig
                                                .info
                                                .keywords
                                                .split(',')
                                                .find_map(|s| {
                                                    let s = s.trim();
                                                    (!s.is_empty()).then(|| s.to_owned())
                                                })
                                                .unwrap_or_default()
                                        } else {
                                            webconfig.info.description.clone()
                                        }
                                    } else {
                                        let domain;
                                        let domain_info;
                                        if tag_name.starts_with("*") {
                                            let domain_info_ = self
                                                .random_domain_info(
                                                    pgsql,
                                                    tag_match.id_str.to_string(),
                                                    &mut id_domain_dict,
                                                    "www",
                                                    "",
                                                )
                                                .await
                                                .unwrap_or_else(|| HashMap::new());
                                            domain_info = domain_info_;
                                            domain = domain_info
                                                .get("full_domain")
                                                .map_or("", |v| v)
                                                .to_string();
                                        } else {
                                            domain = format!(
                                                "www.{}",
                                                req_state.domain_info["root_domain"]
                                            );
                                            domain_info = req_state.domain_info.clone();
                                        }

                                        if domain.is_empty() {
                                            String::new()
                                        } else {
                                            let config_path = format!(
                                                "{}/{}.toml",
                                                domain_info["root_domain"], domain
                                            );

                                            let new_webconfig = if let Some(webconfig) =
                                                webconfig_dict.get(&domain)
                                            {
                                                // println!("{} 配置缓存存在，直接返回", domain);
                                                webconfig.clone()
                                            } else {
                                                // println!("Domain {} not found, fetching...", domain);
                                                match self
                                                    .fetch_or_create_config(
                                                        true,
                                                        config_dict,
                                                        &pgsql,
                                                        &config_path,
                                                        &domain,
                                                    )
                                                    .await
                                                {
                                                    Ok(webconfig) => {
                                                        webconfig_dict.insert(
                                                            domain.clone(),
                                                            webconfig.clone(),
                                                        );
                                                        webconfig
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to fetch config: {}", e);
                                                        return format!("{}?", tag_name);
                                                    }
                                                }
                                            };

                                            let result = if tag_name.ends_with("标题")
                                                || tag_name.ends_with("title")
                                            {
                                                new_webconfig.info.title.clone()
                                            } else if tag_name.ends_with("关键词组")
                                                || tag_name.ends_with("keywords")
                                            {
                                                new_webconfig.info.keywords.clone()
                                            } else if tag_name.ends_with("关键词")
                                                || tag_name.ends_with("keyword")
                                            {
                                                let keywords: Vec<&str> = new_webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .map(|s| s.trim())
                                                    .filter(|s| !s.is_empty())
                                                    .collect();
                                                keywords
                                                    .choose(&mut rand::rng())
                                                    .map(|s| s.to_string())
                                                    .unwrap_or("".to_string())
                                            } else if tag_name.ends_with("核心词")
                                                || tag_name.ends_with("coreword")
                                            {
                                                new_webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .find_map(|s| {
                                                        let s = s.trim();
                                                        (!s.is_empty()).then(|| s.to_owned())
                                                    })
                                                    .unwrap_or_default()
                                            } else {
                                                new_webconfig.info.description.clone()
                                            };

                                            result
                                        }
                                    }
                                }
                                "泛站.域名" | "sub.domain" | "泛站.首页" | "sub.index"
                                | "*泛站.域名" | "*sub.domain" | "*泛站.首页" | "*sub.index" =>
                                {
                                    let subdomain: &str = if tag_name.starts_with("*") {
                                        ""
                                    } else {
                                        &req_state.domain_info["root_domain"]
                                    };
                                    match self
                                        .random_domain_info(
                                            pgsql,
                                            tag_match.id_str.to_string(),
                                            &mut id_domain_dict,
                                            "!www",
                                            subdomain,
                                        )
                                        .await
                                    {
                                        Some(domain_info) => {
                                            if tag_name.ends_with("首页")
                                                || tag_name.ends_with("index")
                                            {
                                                format!("http://{}", domain_info["full_domain"])
                                            } else {
                                                domain_info["full_domain"].to_string()
                                            }
                                        }
                                        None => String::new(),
                                    }
                                }
                                "泛站.网址" | "sub.url" | "*泛站.网址" | "*sub.url" => {
                                    let subdomain: &str = if tag_name.starts_with("*") {
                                        ""
                                    } else {
                                        &req_state.domain_info["root_domain"]
                                    };
                                    match self
                                        .random_domain_info(
                                            pgsql,
                                            tag_match.id_str.to_string(),
                                            &mut id_domain_dict,
                                            "!www",
                                            subdomain,
                                        )
                                        .await
                                    {
                                        Some(domain_info) => {
                                            self.get_domain_url(pgsql, &domain_info["full_domain"])
                                                .await
                                        }
                                        None => String::new(),
                                    }
                                }
                                "泛站.标题"
                                | "泛站.关键词组"
                                | "泛站.关键词"
                                | "泛站.核心词"
                                | "泛站.描述"
                                | "sub.title"
                                | "sub.keywords"
                                | "sub.keyword"
                                | "sub.coreword"
                                | "sub.description"
                                | "*泛站.标题"
                                | "*泛站.关键词组"
                                | "*泛站.关键词"
                                | "*泛站.核心词"
                                | "*泛站.描述"
                                | "*sub.title"
                                | "*sub.keywords"
                                | "*sub.keyword"
                                | "*sub.coreword"
                                | "*sub.description" => {
                                    let subdomain: &str = if tag_name.starts_with("*") {
                                        ""
                                    } else {
                                        &req_state.domain_info["root_domain"]
                                    };
                                    // let Some(domain_info) = self
                                    //     .random_domain_info(
                                    //         pgsql,
                                    //         tag_match.id_str.to_string(),
                                    //         &mut id_domain_dict,
                                    //         "!www",
                                    //         subdomain,
                                    //     )
                                    //     .await
                                    // else {
                                    //     return "泛站.TDK None".to_string();
                                    //     // return String::new();
                                    // };

                                    match self
                                        .random_domain_info(
                                            pgsql,
                                            tag_match.id_str.to_string(),
                                            &mut id_domain_dict,
                                            "!www",
                                            subdomain,
                                        )
                                        .await
                                    {
                                        Some(domain_info) => {
                                            // self.get_domain_url(pgsql, &domain_info["full_domain"])
                                            //     .await

                                            // let domain_info = domain_info_from_domain(&domain);
                                            let config_path = format!(
                                                "{}/{}.toml",
                                                domain_info["root_domain"],
                                                domain_info["full_domain"]
                                            );

                                            let webconfig = if let Some(webconfig) =
                                                webconfig_dict.get(&domain_info["full_domain"])
                                            {
                                                // println!(
                                                //     "{} 配置缓存存在，直接返回",
                                                //     domain_info["full_domain"]
                                                // );
                                                webconfig.clone()
                                            } else {
                                                // println!(
                                                //     "Domain {} not found, fetching...",
                                                //     domain_info["full_domain"]
                                                // );
                                                match self
                                                    .fetch_or_create_config(
                                                        true,
                                                        config_dict,
                                                        &pgsql,
                                                        &config_path,
                                                        &domain_info["full_domain"],
                                                    )
                                                    .await
                                                {
                                                    Ok(webconfig) => {
                                                        webconfig_dict.insert(
                                                            domain_info["full_domain"].clone(),
                                                            webconfig.clone(),
                                                        );
                                                        webconfig
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to fetch config: {}", e);
                                                        return format!("{}?", tag_name);
                                                    }
                                                }
                                            };

                                            let result = if tag_name.ends_with("标题")
                                                || tag_name.ends_with("title")
                                            {
                                                webconfig.info.title.clone()
                                            } else if tag_name.ends_with("关键词组")
                                                || tag_name.ends_with("keywords")
                                            {
                                                webconfig.info.keywords.clone()
                                            } else if tag_name.ends_with("关键词")
                                                || tag_name.ends_with("keyword")
                                            {
                                                let keywords: Vec<&str> = webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .map(|s| s.trim())
                                                    .filter(|s| !s.is_empty())
                                                    .collect();
                                                keywords
                                                    .choose(&mut rand::rng())
                                                    .map(|s| s.to_string())
                                                    .unwrap_or("".to_string())
                                            } else if tag_name.ends_with("核心词")
                                                || tag_name.ends_with("coreword")
                                            {
                                                webconfig
                                                    .info
                                                    .keywords
                                                    .split(',')
                                                    .find_map(|s| {
                                                        let s = s.trim();
                                                        (!s.is_empty()).then(|| s.to_owned())
                                                    })
                                                    .unwrap_or_default()
                                            } else {
                                                webconfig.info.description.clone()
                                            };
                                            result
                                        }
                                        None => String::new(),
                                    }
                                }
                                "路径关键词" | "path_keyword" => {
                                    // URL路径中的最后的一段是中文关键词的部分
                                    let url = &req_state.url;
                                    let mut keyword = "{@keyword}".to_string();
                                    
                                    if let Some(last_segment) = url.rsplit('/').next() {
                                        println!("last_segment: {}", last_segment);
                                        
                                        // 尝试解码URL编码的字符串
                                        if let Ok(decoded) = decode(last_segment) {
                                            let decoded_str = decoded.into_owned();
                                            println!("decoded: {}", decoded_str);
                                            
                                            if decoded_str.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fa5}') {
                                                keyword = decoded_str;
                                            }
                                        } else {
                                            // 如果解码失败，使用原始字符串
                                            if last_segment.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fa5}') {
                                                keyword = last_segment.to_string();
                                            }
                                        }
                                    }
                                    keyword
                                },
                                "时间" => {
                                    // 获取当前日期和时间
                                    let now = Utc::now();
                                    now.format("%Y-%m-%d %H:%M").to_string()
                                },
                                "日期" => {
                                    // 获取当前日期的年月日
                                    let now = Utc::now();
                                    now.format("%Y-%m-%d").to_string()
                                },
                                "仅时间" => {
                                    // 获取当前时间的小时和分钟
                                    let now = Utc::now();
                                    now.format("%H:%M").to_string()
                                },
                                _ => {
                                    if tag_name.starts_with("^") {
                                        let chars_after_caret: Vec<char> =
                                            tag_name[1..].chars().collect();
                                        chars_after_caret
                                            .choose(&mut rand::rng())
                                            .map(|c| c.to_string())
                                            .unwrap_or_else(|| "".to_string())
                                    } else if tag_name.contains("|") {
                                        let t_list: Vec<&str> = tag_name
                                            .split('|')
                                            .map(|s| s.trim())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        t_list
                                            .choose(&mut rand::rng())
                                            .map(|s| s.to_string())
                                            .unwrap_or("".to_string())
                                    } else {
                                        eprintln!("警告: 未定义的固定标签: {}", tag_name);
                                        String::new()
                                    }
                                }
                            };
                        }
                    }
                    _ => eprintln!("错误: 未知标签类型: {}", tag_match.tag_type),
                }
                let duration = start_time.elapsed();
                let haoshi = duration.as_secs() as f64 * 1000.0
                    + duration.subsec_nanos() as f64 / 1_000_000.0;
                if haoshi > 1.0 {
                    eprintln!(
                        "DEBUG: 标签处理耗时: {} (type: {}, id_str: {}) - {:.2?}ms",
                        tag_match.full_match, tag_match.tag_type, tag_match.id_str, haoshi
                    );
                }
            }

            // 3. 按位置顺序应用替换
            tag_stack.sort_by_key(|a| a.start_pos);
            let mut output_chars: Vec<char> = output.chars().collect();
            let mut delta = 0; // 用于跟踪字符位置偏移

            for tag_match in tag_stack {
                if !tag_match.replacement.is_empty() {
                    // eprintln!(
                    // "DEBUG: 替换标签: {} -> {} (id_str: {}, pos: {}-{})",
                    //     tag_match.full_match,
                    //     tag_match.replacement,
                    //     tag_match.id_str,
                    //     tag_match.start_pos,
                    //     tag_match.end_pos
                    // );

                    if tag_match.id_str.is_empty() {
                        // 字符级精确替换（针对无ID的标签）
                        let full_match_chars: Vec<char> = tag_match.full_match.chars().collect();
                        let replacement_chars: Vec<char> = tag_match.replacement.chars().collect();

                        let find_match_pos = |haystack: &[char], needle: &[char]| {
                            haystack
                                .windows(needle.len())
                                .position(|window| window == needle)
                        };

                        if let Some(pos) = find_match_pos(&output_chars[delta..], &full_match_chars)
                        {
                            let start = delta + pos;
                            let end = start + full_match_chars.len();

                            if end <= output_chars.len() {
                                output_chars.splice(start..end, replacement_chars.clone());
                                delta += replacement_chars
                                    .len()
                                    .saturating_sub(full_match_chars.len());
                                changed = true;
                            }
                        }
                    } else {
                        // 全局替换（针对有ID的标签）
                        let old_output = output_chars.iter().collect::<String>();
                        let new_output =
                            old_output.replace(&tag_match.full_match, &tag_match.replacement);

                        if old_output != new_output {
                            output_chars = new_output.chars().collect();
                            delta = 0; // 重置偏移量
                            changed = true;
                        }
                    }
                }
            }

            // 转换回字符串
            output = output_chars.into_iter().collect();

            result = output;
            if !changed {
                break;
            }
        }

        if iteration_count >= MAX_ITERATIONS {
            eprintln!("警告: 达到最大解析迭代次数（可能存在循环嵌套）");
        }

        result
    }

    async fn random_domain_info(
        &self,
        pgsql: &Arc<PgsqlService>,
        id_str: String,
        id_domain_dict: &mut DashMap<String, String>,
        subdomain: &str,
        root_domain: &str,
    ) -> Option<HashMap<String, String>> {
        // 1. 尝试从缓存获取域名
        let domain = if !id_str.is_empty() {
            id_domain_dict.get(&id_str).as_deref().cloned()
        } else {
            None
        };

        // 2. 如果缓存中没有，则随机获取一个网站
        let domain = match domain {
            Some(domain) => {
                // println!("{} id_domain_dict配置缓存存在，直接返回", domain);
                domain
            }
            None => {
                // 从数据库随机获取网站
                match get_random_websites(pgsql, "website_config", subdomain, &root_domain).await {
                    Some(websites) => {
                        // println!("websites: {:?}", websites);
                        if let Some(random_domain) = websites.iter().choose(&mut rand::rng()) {
                            // println!("随机选择域名: {}", random_domain);
                            // 存入缓存
                            if !id_str.is_empty() {
                                id_domain_dict.insert(id_str.clone(), random_domain.clone());
                            }
                            random_domain.clone()
                        } else {
                            println!("网站列表为空");
                            return None;
                        }
                    }
                    None => {
                        println!(
                            "subdomain：{} root_domain：{}未能从 website_config 获取网站列表",
                            subdomain, root_domain
                        );
                        return None;
                    }
                }
            }
        };
        Some(domain_info_from_domain(&domain))
    }

    async fn get_domain_url(&self, pgsql: &Arc<PgsqlService>, domain: &str) -> String {
        let domain_info = domain_info_from_domain(domain);
        let table_name = format!(
            "{}__{}",
            domain_info["subdomain"], domain_info["root_domain"]
        )
        .replace(".", "_");
        match get_cache_urls(pgsql, &table_name, "缓存").await {
            Some(urls) => urls
                .iter()
                .choose(&mut rand::rng())
                .map(|new_link| {
                    // println!("从urls 抽中:{}", new_link);
                    format!("http:{}", new_link) // 使用https更安全
                })
                .unwrap_or_else(|| {
                    println!("缓存urls列表为空");
                    format!("http://{}", domain)
                }),
            None => {
                println!("未能从【{}】获取任何缓存url", table_name);
                format!("http://{}", domain)
            }
        }
    }

    pub fn get_doc_path(&self, doc_name: &str) -> Option<String> {
        // 1. 优先检查文件
        let file_path = Path::new("doc").join(format!("{}.txt", doc_name));
        if file_path.exists() && file_path.is_file() {
            return file_path.to_str().map(|s| s.to_string());
        }

        // 2. 检查目录
        let dir_path = Path::new("doc").join(doc_name);
        if dir_path.exists() && dir_path.is_dir() {
            return self.get_random_filename(&dir_path, Some("txt"), usize::MAX);
        }

        None
    }

    fn parse_params(&self, params_str: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut current_param = String::new();
        let mut in_single_quotes = false; // Track single quote state
        let mut escape = false; // Track escape character state

        let chars: Vec<char> = params_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if escape {
                // Handle escaped characters
                match c {
                    'n' => current_param.push('\n'),
                    't' => current_param.push('\t'),
                    'r' => current_param.push('\r'),
                    '\\' => current_param.push('\\'),
                    '\'' => current_param.push('\''),
                    _ => current_param.push(c), // Pass other escaped chars as-is
                }
                escape = false;
                i += 1;
            } else {
                match c {
                    '\\' => {
                        escape = true;
                        i += 1;
                    }
                    '\'' => {
                        in_single_quotes = !in_single_quotes;
                        current_param.push(c);
                        i += 1;
                    }
                    ',' if !in_single_quotes => {
                        // Split on comma outside single quotes
                        params.push(current_param.trim().to_string());
                        current_param.clear();
                        i += 1;
                    }
                    _ => {
                        current_param.push(c);
                        i += 1;
                    }
                }
            }
        }

        // Add the last parameter
        if !current_param.is_empty() {
            params.push(current_param.trim().to_string());
        }

        params
    }

    pub fn get_random_filename(
        &self,
        dir: &Path,
        suffix: Option<&str>,
        max_depth: usize,
    ) -> Option<String> {
        let files: Vec<String> = WalkDir::new(dir)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let path = e.path();

                // 无后缀要求时直接返回路径
                if suffix.is_none() {
                    return path.to_str().map(|s| s.to_string());
                }

                // 检查扩展名是否匹配
                let matches = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == suffix.unwrap());

                if matches {
                    match path.to_str() {
                        Some(s) => Some(s.to_string()),
                        None => None,
                    }
                } else {
                    None
                }
            })
            .collect();

        files.choose(&mut rand::rng()).cloned()
    }

    /// 单独的函数：获取或生成配置文件
    pub async fn fetch_or_create_config(
        &self,
        is_www: bool,
        config_dict: &Config,
        pgsql: &Arc<PgsqlService>,
        config_path: &str,
        domain: &str,
    ) -> Result<WebsiteConf, StatusCode> {
        let mut conditions: HashMap<&str, &str> = HashMap::new();
        conditions.insert("domain", domain);

        match pgsql
            .fetch_data(
                "website_config",
                &[],
                conditions,
                None,
                Some(1),
                Some(1),
                None,
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
            })
        })
        .collect();
                let website_config: WebsiteConf = match items.into_iter().next() {
                    Some(item) => serde_json::from_value(item).map_err(|e| {
                        eprintln!("Failed to deserialize: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?,
                    None => {
                        // eprintln!("No items found");
                        println!("{} 没有配置文件", domain);
                        // ----------------------------------------------------------
                        if is_www {
                            if !config_dict.website_settings.auto_site_building {
                                println!("{} 自动建站已经关闭", domain);
                                return Err(StatusCode::BAD_REQUEST);
                            }
                        } else {
                            if config_dict.website_settings.auto_site_building
                                && !config_dict.website_settings.pan_site_auto_site_building
                            {
                                println!("{} 泛站自动建站已经关闭", domain);
                                return Err(StatusCode::BAD_REQUEST);
                            }
                        }
                        println!("{} 自动生成配置文件", config_path);
                        // 检测域名归属
                        let name = config_dict.program_info.program_name.clone();
                        let check_url = format!("http://{}/_api/program_name", domain);
                        match self.fetch_url(&check_url).await {
                            Ok((content_type, file_bytes)) => {
                                let text =
                                    self.detect_encoding_and_decode(&file_bytes, &content_type);
                                println!("访问结果:{}\n自己获取:{}", text, name);
                                if text == name {
                                    println!("{} 域名归属正确", domain);
                                } else {
                                    println!("{} 域名归属错误", domain);
                                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                                }
                            }
                            Err(e) => {
                                println!("{} 域名归属错误", domain);
                                return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            }
                        }
                        return Err(StatusCode::NOT_FOUND);
                    }
                };

                Ok(website_config)
            } // 表存在，直接返回数据
            Err(status) => {
                // 报错
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // 判断 client_ip 是否匹配 ip_patterns 中的任意模式
    pub fn hit_ip(&self, ip_patterns: Vec<String>, client_ip: &str) -> bool {
        // 遍历 ip_patterns，检查 client_ip 是否匹配
        ip_patterns.iter().any(|pattern| {
            if pattern.ends_with(".*") {
                // 对于 1.1.1.* 模式，提取前缀（如 1.1.1.）并使用 starts_with 判断
                let prefix = pattern.trim_end_matches("*"); // 去掉尾部的 .*
                client_ip.starts_with(prefix)
            } else {
                // 对于完整 IP 地址，直接比较
                pattern == client_ip
            }
        })
    }
}
