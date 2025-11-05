use crate::my_const::YOUDAOKEY;
use axum::http::StatusCode;
use lol_html::html_content;
// use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use md5::Context;
use rand_user_agent::UserAgent;
// use regex::Regex;
use reqwest;
use std::collections::HashSet;
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
use serde_yaml::Value as YamlValue;

// use std::collections::HashMap;

// use std::collections::HashMap;
use std::str;
use uuid::Uuid;
// use axum::http::StatusCode;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

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
            ("useTerm", "true"),
            ("vendor", "web"),
            ("yduuid", "abcdefg"),
        ];

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
        let iv = md5::compute(
            b"ydsecret://query/iv/C@lZe2YzHtZ2CYgaXKSVfsb7Y4QWHjITPPZ0nQp87fBeJ!Iv6v^6fvi2WN@bYpJ4",
        );
        let key = md5::compute(b"ydsecret://query/key/B*RGygVywfNBwpmBaZg*WT7SIOUP2T0C9WHMZN39j^DAdaZhAnxvGcCY6VYFwnHl");

        let cipher = Aes128Cbc::new_from_slices(&key.0, &iv.0)?;

        let decoded = general_purpose::URL_SAFE.decode(text)?;
        let decrypted = cipher.decrypt_vec(&decoded)?;

        let unpadded_message = str::from_utf8(&decrypted)?;
        let json_data: Value = serde_json::from_str(unpadded_message)?;

        Ok(json_data)
    }

    pub async fn trans_yaml(
        &self,
        yaml_content: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut yaml_value: YamlValue = serde_yaml::from_str(yaml_content)?;
        let mut texts_to_translate = Vec::new();

        self.extract_texts_to_translate(&mut yaml_value, &mut texts_to_translate);

        

        let translations = self
            .trans_batch_sorted(texts_to_translate, from, to, use_ip)
            .await?;

        self.replace_translations(&mut yaml_value, &translations);

        Ok(serde_yaml::to_string(&yaml_value)?.replace("'''", "'"))
    }

    pub async fn trans_html(
        &self,
        html_content: &str,
        yaml_content: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut yaml_value: YamlValue = serde_yaml::from_str(yaml_content)?;
        let mut texts_to_translate = Vec::new();

        self.extract_texts_to_translate(&mut yaml_value, &mut texts_to_translate);
        // println!("Texts to translate: {:?}", texts_to_translate);

        let translations = self
            .trans_batch_sorted(texts_to_translate, from, to, use_ip)
            .await?;

        // println!("Translations: {:?}", translations);
        if translations.is_empty() {
            return Err("No translations found".into());
        }

        let replaced_html = self.replace_html(html_content, &translations)?;

        Ok(replaced_html)
    }

    fn replace_html(
        &self,
        html: &str,
        translations: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use html_escape::encode_text;

        // 1. 预处理：按长度排序并编码
        let mut sorted: Vec<_> = translations
            .iter()
            .map(|(k, v)| (encode_text(k).into_owned(), encode_text(v).into_owned()))
            .collect();

        // 从长到短排序以确保优先替换长的文本
        sorted.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()));

        // println!("Sorted translations: {:?}", sorted);

        // 2. 逐步替换
        let mut result = html.to_string().replace("&nbsp;", " ").replace(" ", " ");
        for (encoded_key, encoded_value) in sorted {
            // 处理可能的转义差异
            let variations = [
                &encoded_key,
                &encoded_key.replace("&#39;", "&apos;"), // 处理单引号的不同转义
                &encoded_key.replace("&quot;", "&#34;"), // 处理双引号的不同转义
            ];

            for variant in variations {
                if result.contains(variant) {
                    result = result.replace(variant, &encoded_value);
                    break;
                }
            }
        }

        Ok(result)
    }

    fn extract_texts_to_translate(
        &self,
        yaml_value: &mut YamlValue,
        texts_to_translate: &mut Vec<String>,
    ) {
        let mut stack = vec![yaml_value];

        while let Some(current) = stack.pop() {
            match current {
                YamlValue::String(s) => {
                    if !s.trim().is_empty() {
                        texts_to_translate.push(s.trim_matches('\'').to_string());
                    }
                }
                YamlValue::Mapping(m) => {
                    for (_, v) in m.iter_mut() {
                        stack.push(v);
                    }
                }
                YamlValue::Sequence(s) => {
                    for v in s.iter_mut() {
                        stack.push(v);
                    }
                }
                _ => {}
            }
        }
    }

    fn replace_translations(
        &self,
        yaml_value: &mut YamlValue,
        translations: &HashMap<String, String>,
    ) {
        let mut stack = vec![yaml_value];

        // Sort translation keys by length in descending order
        let mut sorted_translations: Vec<_> = translations.iter().collect();
        sorted_translations.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        while let Some(current) = stack.pop() {
            match current {
                YamlValue::String(s) => {
                    let mut result = String::new();
                    let mut chars = s.chars(); // 获取字符迭代器
                    let mut i = 0;

                    while i < s.len() {
                        let mut matched = false;
                        for (key, value) in &sorted_translations {
                            if s[i..].starts_with(key.as_str()) {
                                result.push_str(value);
                                i += key.len();
                                // 同步推进字符迭代器
                                for _ in 0..key.chars().count() {
                                    chars.next();
                                }
                                matched = true;
                                break;
                            }
                        }

                        if !matched {
                            if let Some(c) = chars.next() {
                                result.push(c);
                            }
                            i += 1;
                        }
                    }
                    *s = format!("'{}'", result.trim_matches('“').trim_matches('”'));
                }
                YamlValue::Mapping(m) => {
                    for (_, v) in m.iter_mut() {
                        stack.push(v);
                    }
                }
                YamlValue::Sequence(s) => {
                    for v in s.iter_mut() {
                        stack.push(v);
                    }
                }
                _ => {}
            }
        }
    }

    async fn trans_multi(
        &self,
        texts: Vec<String>,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let from_lang = if from == "zh" { "zh-CHS" } else { from };
        let to_lang = if to == "zh" { "zh-CHS" } else { to };

        let unique_texts: Vec<String> = texts
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();
        let json_data = self
            .trans_resp_multi(&unique_texts.join("\n"), from_lang, to_lang, use_ip)
            .await?;
        let mut result = HashMap::new();

        if let Some(results) = json_data["translateResult"].as_array() {
            for item in results {
                if let Some(entries) = item.as_array() {
                    for entry in entries {
                        if let (Some(src), Some(tgt)) =
                            (entry["src"].as_str(), entry["tgt"].as_str())
                        {
                            result.insert(src.trim().to_string(), tgt.trim().to_string());
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    async fn trans_resp_multi(
        &self,
        word: &str,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
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
        let iv = md5::compute(
            b"ydsecret://query/iv/C@lZe2YzHtZ2CYgaXKSVfsb7Y4QWHjITPPZ0nQp87fBeJ!Iv6v^6fvi2WN@bYpJ4",
        );
        let key = md5::compute(b"ydsecret://query/key/B*RGygVywfNBwpmBaZg*WT7SIOUP2T0C9WHMZN39j^DAdaZhAnxvGcCY6VYFwnHl");

        let cipher = Aes128Cbc::new_from_slices(&key.0, &iv.0)?;

        let decoded = general_purpose::URL_SAFE.decode(text)?;
        let decrypted = cipher.decrypt_vec(&decoded)?;

        let unpadded_message = str::from_utf8(&decrypted)?;
        let json_data: Value = serde_json::from_str(unpadded_message)?;

        // println!("Response: {:?}", json_data);

        Ok(json_data)
    }

    async fn trans_batch_sorted(
        &self,
        texts: Vec<String>,
        from: &str,
        to: &str,
        use_ip: &str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let translations = self.trans_multi(texts.clone(), from, to, use_ip).await?;
        let mut sorted_translations: Vec<(String, String)> = translations
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        sorted_translations.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let mut final_result = HashMap::new();
        for (text, translation) in sorted_translations {
            final_result.insert(text, translation);
        }
        // println!("Result: {:?}", final_result);

        Ok(final_result)
    }
}
