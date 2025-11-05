use crate::my_const::YOUDAOKEY;
use axum::http::StatusCode;
// use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use md5::Context;
use rand_user_agent::UserAgent;
// use regex::Regex;
use reqwest;
// use reqwest::header::HeaderValue;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
// use urlencoding::decode;
use aes::Aes128;
// use base64;
use base64::{engine::general_purpose, Engine as _};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use md5;
// use rand::Rng;
use urlencoding::encode;
// use reqwest::Client;
use serde_json::Value;
// use std::collections::HashMap;
use std::str;
use uuid::Uuid;
// use axum::http::StatusCode;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// #[derive(Debug)]
// struct WebTransResult {
//     success: bool,
//     from: String,
//     to: String,
//     source: String,
// }

pub struct YouDao {}

impl YouDao {
    pub async fn trans_resp(
        &self,
        word: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        // let t = "Vy4EQ1uwPkUoqvcP1nIu6WiAjxFeA3Y2";
        let p = format!(
            "client=fanyideskweb&mysticTime={}&product=webfanyi&key={}",
            t, YOUDAOKEY
        );
        let sign = format!("{:x}", md5::compute(p.as_bytes()));

        let form = [
            ("abtest", "0"),
            ("appVersion", "1.0.0"),
            ("client", "fanyideskweb"),
            ("dictResult", "false"),
            ("domain", "0"),
            ("from", from),
            ("i", word),
            ("keyfrom", "fanyi.web"),
            ("keyid", "webfanyi"),
            ("mid", "1"),
            ("model", "1"),
            ("mysticTime", &t.to_string()),
            ("network", "wifi"),
            ("pointParam", "client,mysticTime,product"),
            ("product", "webfanyi"),
            ("screen", "1"),
            ("sign", &sign),
            ("to", to),
            ("useTerm", "false"),
            ("vendor", "web"),
            ("yduuid", "abcdefg"),
        ];

        // let mut rng = rand::rng();
        // let user_id: i64 = rng.random_range(1000000000..=9999999999); // 使用 i64 类型

        // let ip_address = format!(
        //     "{}.{}.{}.{}",
        //     rng.random_range(0..255),
        //     rng.random_range(0..255),
        //     rng.random_range(0..255),
        //     rng.random_range(0..255)
        // );
        // let outfox_search_user_id = format!("{}@{}", user_id, ip_address)-1371528901@199.182.234.92;
        //359728185@199.182.234.103
        let outfox_search_user_id = format!("{}@{}", "359728185", "199.182.234.103");
        let uetvid = Uuid::new_v4().to_string();
        let cookie = format!(
            "OUTFOX_SEARCH_USER_ID={}; _uetvid={}",
            outfox_search_user_id, uetvid
        );

        let ip = IpAddr::from_str(use_ip)?;
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .local_address(ip)
            .build()?;
        let rua = UserAgent::pc().to_string();

        let response = client
            .post("https://dict.youdao.com/webtranslate")
            .form(&form)
            .header("User-Agent", rua)
            .header("Accept", "application/json, text/plain, */*")
            .header(
                "Accept-Language",
                "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6",
            )
            .header("Connection", "keep-alive")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", &cookie)
            .header("Host", "dict.youdao.com")
            .header("Origin", "https://fanyi.youdao.com")
            .header("Referer", "https://fanyi.youdao.com/")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-site")
            .header(
                "Sec-Ch-Ua",
                "\"Not A(Brand\";v=\"8\", \"Chromium\";v=\"132\", \"Microsoft Edge\";v=\"132\"",
            )
            .header("Sec-Ch-Ua-Mobile", "?0")
            .header("Sec-Ch-Ua-Platform", "\"Windows\"")
            .send()
            .await?;

        let text = response.text().await?;
        // let text = "www.baidu.com";
        let iv = md5::compute(
            b"ydsecret://query/iv/C@lZe2YzHtZ2CYgaXKSVfsb7Y4QWHjITPPZ0nQp87fBeJ!Iv6v^6fvi2WN@bYpJ4",
        );
        let key = md5::compute(b"ydsecret://query/key/B*RGygVywfNBwpmBaZg*WT7SIOUP2T0C9WHMZN39j^DAdaZhAnxvGcCY6VYFwnHl");

        let cipher = Aes128Cbc::new_from_slices(&key.0, &iv.0)?;

        // let decoded = base64::decode_config(text, base64::alphabet::URL_SAFE)?;
        let decoded = general_purpose::URL_SAFE.decode(text)?;
        // let decoded = base64::engine::general_purpose::URL_SAFE.decode(text).ok();
        let decrypted = cipher.decrypt_vec(&decoded)?;

        let unpadded_message = str::from_utf8(&decrypted)?;
        let json_data: Value = serde_json::from_str(unpadded_message)?;

        // let text = response.text().await?;
        Ok(json_data)

        // Ok(unpadded_message.to_string())
    }

    pub async fn web_trans_resp(
        &self,
        url: &str,
        use_ip: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // println!("翻译网页：{} 使用IP：{}", url, use_ip);
        let ip = IpAddr::from_str(use_ip)?;
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .local_address(ip)
            .build()?;
        let rua = UserAgent::pc().to_string();
        let response = client
            .get(url)
            .header("User-Agent", rua)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header("Connection", "keep-alive")
            .header("Cookie", "OUTFOX_SEARCH_USER_ID_NCOO=1049708127.9005697")
            .header("Host", "webtrans.yodao.com")
            .header("Referer", "http://webtrans.yodao.com/webTransPc/index.html")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await?;
        if !response.status().is_success() {
            println!("HTTP error: {}", response.status());
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        let text = response.text().await?;
        Ok(text)
    }

    pub async fn web_trans(
        &self,
        target_url: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<(String, String), StatusCode> {
        let from_lang = if from == "zh" { "zh-CHS" } else { from };
        let to_lang = if to == "zh" { "zh-CHS" } else { to };

        // 获取当前时间戳
        let o = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? // 如果时间获取失败，返回 500 错误
            .as_millis()
            .to_string();

        let r = "ydsecret://mdictweb.si/sign/-eV=8}L$s4$nPL00op3p]";

        // 处理 URL 参数
        let t = if target_url.contains("&") {
            let parts: Vec<&str> = target_url.splitn(2, '?').collect();
            let encoded = encode(&parts[1]); // 假设 encode 函数已实现
            format!("{}?{}", parts[0], encoded)
        } else {
            target_url.to_string()
        };

        // 计算 MD5
        let s = "mdictweb";
        let c = format!("{}{}{}{}", s, o, r, target_url);
        let mut hasher = Context::new();
        hasher.consume(c.as_bytes());
        let c_md5 = format!("{:x}", hasher.compute());

        // 构造请求 URL
        let url = format!(
            "http://webtrans.yodao.com/server/webtrans/tranUrl?url={}&from={}&to={}&type=1&product=mdictweb&salt={}&sign={}",
            t, from_lang, to_lang, o, c_md5
        );

        println!("翻译网页：{} 使用IP：{}", url, use_ip);

        let mut retry_count = 0;
        while retry_count < 2 {
            match self.web_trans_resp(&url, use_ip).await {
                Ok(resp) => {
                    // println!("Response: {:?}", resp);
                    let source = resp.replace("&amp;", "&");

                    // 检查是否返回了错误页面
                    if source.contains("很抱歉，您输入的网址不存在或当前无法访问，请稍后重试")
                    {
                        if retry_count >= 1 {
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                        retry_count += 1;
                        println!(
                            "Translation failed: Retry {} for {}",
                            retry_count, target_url
                        );
                        continue;
                    }

                    // 解析 HTML 文档
                    let document = Html::parse_document(&source);
                    let title_selector = Selector::parse("title").unwrap();
                    let title = document
                        .select(&title_selector)
                        .next()
                        .map(|e| e.inner_html())
                        .unwrap_or("".to_string());

                    // println!("title:{}", title);
                    if retry_count >= 1
                        && (title.contains("404")
                            || title.contains("异常访问")
                            || title.contains("no found")
                            || title.contains("not found")
                            || source.contains("页面已被删除")
                            || source.contains("页面未找到")
                            || source.contains("输入的地址不正确")
                            || source.contains("403 Forbidden"))
                    {
                        println!("Translation failed: {} 404 not found", target_url);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }

                    // 提取 <meta name="keywords">
                    let keywords_selector = Selector::parse("meta[name='keywords']").unwrap();
                    let keywords = document
                        .select(&keywords_selector)
                        .next()
                        .and_then(|e| e.value().attr("content"))
                        .map(|s| s.to_string())
                        .unwrap_or("".to_string());

                    // 提取 <meta name="description">
                    let description_selector = Selector::parse("meta[name='description']").unwrap();
                    let description = document
                        .select(&description_selector)
                        .next()
                        .and_then(|e| e.value().attr("content"))
                        .map(|s| s.to_string())
                        .unwrap_or("".to_string());

                    // 翻译TDK
                    let tdk = format!(
                        "{}{}{}",
                        if title.len() > 1 {
                            format!("{}\n", title)
                        } else {
                            String::new()
                        },
                        if keywords.len() > 1 {
                            format!("{}\n", keywords)
                        } else {
                            String::new()
                        },
                        if description.len() > 1 {
                            format!("{}\n", description)
                        } else {
                            String::new()
                        },
                    )
                    .trim_end_matches("\n")
                    .to_string();
                    // match self.trans(
                    //                 "this is apple\nhello girl!",
                    //                 "en",
                    //                 "zh-CHS",
                    //                 "0.0.0.0",
                    //             )
                    //             .await
                    //         {
                    //             Ok(transed_text) => {
                    //                 // text = transed_text;
                    //                 println!("翻译测试：{:?}",transed_text);
                    //                 // trans_mode = true;
                    //             }
                    //             Err(e) => {
                    //                 // return Err(e);
                    //                 // 记录错误日志
                    //                 println!("Translation failed: {}", e);
                    //                 // 可以选择重试或返回自定义错误
                    //                 // return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    //             }
                    //         }

                    // match self
                    //     .trans("this is apple", from_lang, to_lang, use_ip)
                    //     .await
                    // {
                    //     Ok(transed_text) => {
                    //         // text = transed_text;
                    //         println!("翻译测试：{:?}", transed_text);
                    //         // trans_mode = true;
                    //     }
                    //     Err(e) => {
                    //         // return Err(e);
                    //         // 记录错误日志
                    //         println!("Translation failed: {}", e);
                    //         // 可以选择重试或返回自定义错误
                    //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    //     }
                    // }
                    // println!("tdk:{}", tdk);
                    // 返回成功的结果
                    return Ok((source, tdk));
                }
                Err(e) => {
                    println!("Error sending request: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }

        // 如果重试次数用尽，返回默认错误
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 翻译方法
    pub async fn trans(
        &self,
        word: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<HashMap<String, String>, StatusCode> {
        let from_lang = if from == "zh" { "zh-CHS" } else { from };
        let to_lang = if to == "zh" { "zh-CHS" } else { to };
        let mut result = HashMap::new();
        // println!("翻译文本：{} 使用IP：{}", word, use_ip);
        match self.trans_resp(word, from_lang, to_lang, use_ip).await {
            Ok(json_data) => {
                // println!(
                //     "json_data {} {} {} {}",
                //     json_data, from_lang, to_lang, use_ip
                // );
                for i in json_data["translateResult"].as_array().unwrap() {
                    for i_dict in i.as_array().unwrap() {
                        let src = i_dict["src"].as_str().unwrap().trim();
                        let tgt = i_dict["tgt"].as_str().unwrap().trim();
                        result.insert(src.to_string(), tgt.to_string());
                    }
                    // let src = i["src"].as_str().unwrap().trim();
                    // let tgt = i["tgt"].as_str().unwrap().trim();
                    // result.insert(src.to_string(), tgt.to_string());
                }
                // println!("翻译结果：{:?}", result);
                Ok(result)
            }
            Err(e) => {
                println!("Error sending request: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}
