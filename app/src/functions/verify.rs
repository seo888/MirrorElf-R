use crate::my_const::{SALT};
use reqwest::Client;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_128_GCM};
use ring::digest;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
// use std::fs;
use rand_user_agent::UserAgent;
// use chrono::{Local, Duration, NaiveDate};
use base64::engine::general_purpose;
use base64::Engine as _;
use chrono::{Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};

pub struct Verify;

impl Verify {
    pub fn new() -> Self {
        Verify
    }

    pub async fn get_my_ip(&self) -> Option<String> {
        // 设置 User-Agent
        let rua = UserAgent::pc().to_string();
    
        // 创建 Client，禁用证书验证
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // 禁用证书验证
            .build()
            .ok()?;
    
        // 发送 GET 请求
        let resp = client
            .get("https://icanhazip.com")
            .header("User-Agent", rua)
            .send()
            .await;
    
        // 处理响应或错误
        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    // 读取响应体
                    let body = response.text().await.ok()?;
                    Some(body.trim().to_string())
                } else {
                    // 打印错误状态码
                    eprintln!("Request failed with status: {}", response.status());
                    None
                }
            }
            Err(err) => {
                // 打印错误信息
                eprintln!("Request failed with error: {}", err);
                None
            }
        }
    }

    pub async fn get_machine_id(&self) -> Option<String> {
        let my_ip = self.get_my_ip().await?;
        // let my_ip = "23.97.62.121";
        println!("my_ip:{}", my_ip);
        let key = hmac::Key::new(hmac::HMAC_SHA256, SALT.as_bytes());
        let result = hmac::sign(&key, my_ip.as_bytes());
        let machine_id = hex::encode(result.as_ref());
        Some(machine_id[43..].to_string())
    }

    pub async fn encrypt_data(&self, expiry_date: &str) -> Option<String> {
        let machine_id = self.get_machine_id().await?;
        println!("machine_id:{}", machine_id);

        let expiry_date = if expiry_date.chars().all(|c| c.is_ascii_digit()) {
            let days = expiry_date.parse::<i64>().ok()?;
            let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
            let now = Utc::now().with_timezone(&beijing_offset);
            now.checked_add_signed(Duration::days(days))?
                .format("%Y/%m/%d %H:%M:%S")
                .to_string()
        } else {
            expiry_date.to_string()
        };

        let data = format!("{}:{}", machine_id, expiry_date);

        // 生成AES密钥
        let salt_digest = digest::digest(&digest::SHA256, SALT.as_bytes());
        let key_bytes = &salt_digest.as_ref()[0..16];
        let unbound_key = UnboundKey::new(&AES_128_GCM, key_bytes).ok()?;
        let key = LessSafeKey::new(unbound_key);

        // 生成随机Nonce
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes).ok()?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut buffer = data.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut buffer)
            .ok()?;

        // 组合Nonce和密文
        let mut ciphertext = nonce_bytes.to_vec();
        ciphertext.extend_from_slice(&buffer);

        let encoded = general_purpose::URL_SAFE.encode(&ciphertext);
        Some(encoded)
    }

    pub async fn decrypt_data(
        &self,
        iv_ct: &str,
        machineid: Option<String>,
    ) -> Result<String, String> {
        let decoded = general_purpose::URL_SAFE
            .decode(iv_ct)
            .map_err(|_| "授权码验证失败，Base64解码失败".to_string())?;
        if decoded.len() < 12 {
            return Err("授权码验证失败，解码数据过短".to_string());
        }
        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes
                .try_into()
                .map_err(|_| "授权码验证失败，Nonce转换失败".to_string())?,
        );

        // 生成AES密钥
        let salt_digest = digest::digest(&digest::SHA256, SALT.as_bytes());
        let key_bytes = &salt_digest.as_ref()[0..16];
        let unbound_key = UnboundKey::new(&AES_128_GCM, key_bytes)
            .map_err(|_| "授权码验证失败，密钥创建失败".to_string())?;
        let key = LessSafeKey::new(unbound_key);

        let mut buffer = ciphertext.to_vec();
        let decrypted = key
            .open_in_place(nonce, Aad::empty(), &mut buffer)
            .map_err(|_| "授权码验证失败，解密失败".to_string())?;
        let result = String::from_utf8(decrypted.to_vec())
            .map_err(|_| "授权码验证失败，UTF-8转换失败".to_string())?;

        // println!("{:?}", result);
        // 使用 split_once 替代 split
        let parts = result.split_once(':');
        let (machine_id_part, expiry_date_part) = match parts {
            Some((machine_id, expiry_date)) => (machine_id, expiry_date),
            None => return Err("授权码验证失败，格式无效".to_string()),
        };

        let machine_id = match machineid {
            Some(id) => {
                // println!("直接用传来的machine_id：{}",id);
                id
            },
            None => {
                // 调用 get_machine_id 并处理可能的错误
                self.get_machine_id().await
                    .ok_or("授权码验证失败，获取机器码失败".to_string())?
            }
        };

        if machine_id_part != machine_id {
            return Err(format!(
                "授权码验证失败，您的机器码为{}，此授权码仅限服务器{}使用",
                machine_id, machine_id_part
            ));
        }

        // 解析北京时间
        let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = Utc::now().with_timezone(&beijing_offset).naive_local();
        let expiry_date = NaiveDateTime::parse_from_str(expiry_date_part, "%Y/%m/%d %H:%M:%S")
            .map_err(|_| "授权码验证失败，解析过期时间失败".to_string())?;

        let remaining = expiry_date - now;

        if remaining < Duration::zero() {
            return Err(format!(
                "授权码验证失败，此授权码已于{}过期",
                expiry_date.format("%Y/%m/%d %H:%M:%S")
            ));
        }

        // 计算剩余时间
        let total_seconds = remaining.num_seconds();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        Ok(format!(
            "授权码将于{}过期（剩余{}天{}小时{}分{}秒）",
            expiry_date.format("%Y/%m/%d %H:%M:%S"),
            days,
            hours,
            minutes,
            seconds
        ))
    }
}
// #[tokio::main]
// async fn main() -> Result<(), Unspecified> {
//     let verify = Verify::new();

//     if let Some(ok_key) = verify.encrypt_data("30").await { // 30天后过期
//         println!("生成授权码: {}", ok_key);
//         if let Some(decrypted_data) = verify.decrypt_data(&ok_key).await {
//             println!("解密数据: {}", decrypted_data);
//         }
//     }
//     Ok(())
// }
