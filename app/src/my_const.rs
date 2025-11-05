// my_const.rs
use indexmap::IndexMap;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;

pub const VERSION: &str = "0.8.1";
pub const SECRET: &str = "Mirror-Elf-R888888";
pub const SALT: &str = "Mirror-Elf888888";
// pub const IP_TXT: &str = "ip.txt";
pub const CONFIG_FILE_PATH: &str = "config/config.yml";
pub const CHINA_JSON_PATH: &str = "config/china.json";
pub const REPALCE_CONTENT: &str = "全局替换:\n  - '待替换字符串 -> {关键词}'\n首页替换:\n  - '待替换字符串 -> {关键词2}'\n内页替换:\n  - '待替换字符串 -> 替换词'";
pub const AD_JS_CODE: &str = r#"<script src="/_static/ad.js"></script>"#;
pub const VERSION_URL: &str = "https://api.github.com/repos/seo888/MirrorElf-R/releases/latest";
pub const YOUDAOKEY: &str = "SRz6r3IGA6lj9i5zW0OYqgVZOtLDQe3e";
pub const CACHE_PAGE_SUFFIX: [&str; 9] = [
    ".php", ".asp", ".jsp", ".aspx", ".jspx", ".html", ".htm", ".shtml", ".xhtml",
];

pub static TITLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<title\b[^>]*>(.*?)</title>").unwrap());
pub static HEAD_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<head\b[^>]*>").unwrap());
pub static HEAD_FOOTER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)</head>").unwrap());
pub static BODY_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<body\b[^>]*>").unwrap());
pub static BODY_FOOTER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)</body>").unwrap());
pub static KUO_HAO_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{([^{}]*)\}").unwrap());
pub static TIMESTAMP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\d+}").unwrap());

// 匹配固定标签 {tag} 或 {tag#id}
// pub static FIXED_TAG_REGEX: Lazy<Regex> =
//     Lazy::new(|| Regex::new(r"\{([^\%\n\r{}#][^\n\r{}#]*)(?:#(\d+))?\}").unwrap());
pub static FIXED_TAG_REGEX: Lazy<Regex> =
    // Lazy::new(|| Regex::new(r"\{([^%\n\r{}#;][^\n\r{}#]*[^;])(?:#(\d+))?\}").unwrap());
    Lazy::new(|| Regex::new(r"\{([^%\n\r{}#;][\p{Han}a-zA-Z0-9_\.|]*[^;])(?:#(\d+))?\}").unwrap());

pub static FUNC_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\!([\p{Han}a-zA-Z_][^\n\r{}()#]*)\(([^()\{\}]*)\)(?:#(\d+))?\}").unwrap()
});

pub static DOC_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{(:?\^?@)([\p{Han}a-zA-Z0-9_][^\n\r{}()#]*)(?:\(([^()\{\}]*)\))?(?:#(\d+))?\}")
        .unwrap()
});

pub const IPV4BIN: &str = "config/IP2LOCATION-LITE-DB3.BIN";

lazy_static! {
    pub static ref SPIDERS_DICT: IndexMap<&'static str, Vec<&'static str>> = {
        let mut map = IndexMap::new();
        map.insert("百度蜘蛛", vec!["baidu", "Baidu"]);
        map.insert("搜狗蜘蛛", vec!["sogou", "Sogou"]);
        map.insert("神马蜘蛛", vec!["Yisou"]);
        map.insert("头条蜘蛛", vec!["Bytespider"]);
        map.insert("必应蜘蛛", vec!["bingbot", "Bingbot"]);
        map.insert("360蜘蛛", vec!["360Spider"]);
        map.insert("谷歌图片蜘蛛", vec!["Googlebot-Image"]);
        map.insert("谷歌蜘蛛", vec!["Googlebot"]);
        map.insert("夸克蜘蛛", vec!["Quark"]);
        map.insert("雅虎蜘蛛", vec!["Yahoo"]);
        map.insert("其它蜘蛛", vec!["spider", "bot"]);
        map.insert("普通用户", vec![""]);
        map
    };
}

pub const SEARCH_URLS: [&str; 9] = [
    ".baidu.com/",
    ".sogou.com/",
    ".sm.cn/",
    ".toutiao.com/",
    ".bing.com/",
    ".so.com/",
    ".google.com/",
    ".quark.cn/",
    ".yahoo.com/",
    // ".github.dev/",
];

pub const PROMPT_TRANS: &str = r#"# 任务概述

**目标**：根据提供的YAML数据，翻译CSS选择器对应的内容，最终返回必需保留输入的YAML的所有键（一个都不能少）。

## 输入数据
- **格式**：YAML对象
  - **键**：CSS选择器
  - **值**：待翻译文本

## 翻译语言
  - **from**：{from}
  - **to**：{to}

## 返回格式
- **结构要求**：
  - 保留输入YAML的所有键。
  - 将值替换为翻译的内容，并用单引号包裹起来。
  - 请直接返回YAML数据，不要有其它文本。"#;
