use linecache::AsyncLineCache;

use crate::my_const::{DOC_TAG_REGEX, FIXED_TAG_REGEX, FUNC_TAG_REGEX};
use crate::{RequestState, WebsiteConf};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MTag<'a> {
    source: String,
    webconfig: WebsiteConf,
    req_state: RequestState, // 固定标签
    linecache: &'a Arc<AsyncLineCache>,
}

impl<'a> MTag<'a> {
    pub fn new(source: String, req_state: RequestState, webconfig: WebsiteConf, linecache: &'a Arc<AsyncLineCache>,) -> Self {
        Self {
            source,
            req_state,
            webconfig,
            linecache
        }
    }

    /// 主解析入口（支持嵌套）
    pub async fn parse(&self) -> String {
        let mut result = self.source.clone();
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 10; // 防止无限循环

        loop {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                eprintln!("警告：达到最大解析迭代次数（可能存在循环嵌套）");
                break;
            }

            let (changed, new_result) = self.parse_internal(&result).await;
            result = new_result;
            if !changed {
                break;
            }
        }

        result
    }

    async fn parse_internal(&self, input: &str) -> (bool, String) {
        let mut changed = false;
        let mut output = input.to_string();

        // 1. 先解析函数标签（支持嵌套）
        if let Some(caps) = FUNC_TAG_REGEX.captures(&output) {
            let full_match = caps.get(0).unwrap().as_str();
            let func_name = caps.get(1).unwrap().as_str();
            let params_str = caps.get(2).unwrap().as_str();

            // // 递归解析参数中的嵌套标签
            // let parsed_params = if params_str.contains('{') {
            //     let nested_parser = MTag::new(
            //         params_str.to_string(),
            //         self.req_state.clone(),
            //         self.webconfig.clone(),
            //         self.linecache,
            //     );
            //     nested_parser.parse().await
            // } else {
            //     params_str.to_string();
            // };
            let parsed_params = params_str.to_string();

            let replacement = self.parse_func_tag(func_name, &parsed_params);
            output = output.replace(full_match, &replacement);
            changed = true;
        }

        // 2. 解析文档标签
        if let Some(caps) = DOC_TAG_REGEX.captures(&output) {
            let full_match = caps.get(0).unwrap().as_str();
            let doc_name = caps.get(1).unwrap().as_str();
            let format = caps.get(2).map(|m| m.as_str());

            let replacement = self.parse_doc_tag(doc_name, format).await;
            output = output.replace(full_match, &replacement);
            changed = true;
        }

        // 3. 最后解析固定标签（避免与函数/文档标签冲突）
        let mut replacements = Vec::new();
        for cap in FIXED_TAG_REGEX.captures_iter(&output) {
            let full_match = cap.get(0).unwrap().as_str().to_string();
            let tag_name = cap.get(1).unwrap().as_str();
            let replacement = self.parse_fixed_tag(tag_name);
            replacements.push((full_match, replacement));
        }
        for (full_match, replacement) in replacements {
            output = output.replace(&full_match, &replacement);
            changed = true;
        }

        (changed, output)
    }

    // --- 标签类型解析方法 ---

    // 解析固定标签
    fn parse_fixed_tag(&self, tag: &str) -> String {
        match tag {
            "网址" | "url" => {
                // 当前页面地址
                self.req_state.url.clone()
            }
            "首页网址" | "index_url" => {
                // 网站首页地址
                let url = &self.req_state.url;
                // 提取协议和主域名部分
                if let Some(scheme_end) = url.find("://") {
                    let domain_start = scheme_end + 3;
                    if let Some(path_start) = url[domain_start..].find('/') {
                        return url[..domain_start + path_start].to_string();
                    }
                }
                url.clone() // 如果不符合URL格式则返回原样
            }
            "标题" | "title" => {
                // 站点标题
                self.webconfig.info.title.clone()
            }
            "关键词" | "keywords" => {
                // 站点关键词
                self.webconfig.info.keywords.clone()
            }
            "核心词" | "keyword" => {
                // 站点关键词第一个
                self.webconfig
                    .info
                    .keywords
                    .split(',')
                    .next()
                    .map(|s| s.trim()) // 去除前后空格
                    .filter(|s| !s.is_empty()) // 过滤空字符串
                    .unwrap_or("") // 默认值
                    .to_string()
            }
            "描述" | "description" => {
                // 站点描述
                self.webconfig.info.description.clone()
            }
            "路径关键词" | "path_keyword" => {
                // URL路径中的最后的一段是中文关键词的部分
                let url = &self.req_state.url;
                if let Some(last_segment) = url.rsplit('/').next() {
                    // 简单判断是否包含中文字符
                    if last_segment.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fa5}') {
                        return last_segment.to_string();
                    }
                }
                "{@keyword}".to_string()
            },
            "当前关键词" => "站点关键词".to_string(),
            "当前核心词" => "站点关键词第一个".to_string(),
            "当前描述" => "站点描述".to_string(),
            _ => format!("未定义的固定标签: {}", tag),
        }
    }

    fn parse_func_tag(&self, func_name: &str, params_str: &str) -> String {
        let params: Vec<String> = params_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        match func_name {
            // 文件操作
            "GetFileName" => self.get_file_name(&params),
            // "GetFileSize" => self.get_file_size(&params),

            // 字符串处理
            "ToUpper" => params.get(0).map_or("".into(), |s| s.to_uppercase()),
            "Substring" => self.substring(&params),

            // 数学运算
            "Add" => self.math_operation(&params, |a, b| a + b),

            _ => format!("{{$未定义函数: {}}}", func_name),
        }
    }

    async fn parse_doc_tag(&self, doc_type: &str, format: Option<&str>) -> String {
        let tag_path = format!("doc/{}.txt", doc_type);
        match self.linecache.random_line(&tag_path).await {
            Ok(Some(line)) => {
                line
            }
            _ => {
                println!("无法获取 tag_path: {}", tag_path);
                "".to_string()
            }
        }
        // match (doc_type, format.unwrap_or("default")) {
        //     ("title", "chinese") => "中文标题".to_string(),
        //     ("date", "short") => "2023-11-15".to_string(),
        //     _ => format!("{{@未定义文档: {}/{}}}", doc_type, format.unwrap_or("")),
        // }
    }

    // --- 函数实现 ---

    fn get_file_name(&self, params: &[String]) -> String {
        params
            .get(0)
            .map(|path| {
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("invalid_path")
                    .to_string()
            })
            .unwrap_or_else(|| "default.txt".into())
    }

    fn substring(&self, params: &[String]) -> String {
        if params.len() < 3 {
            return "参数不足".into();
        }

        let s = &params[0];
        let start = params[1].parse().unwrap_or(0);
        let len = params[2].parse().unwrap_or(s.len());

        s.chars().skip(start).take(len).collect()
    }

    fn math_operation<F>(&self, params: &[String], op: F) -> String
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut result = 0.0;
        for param in params {
            if let Ok(num) = param.parse::<f64>() {
                result = op(result, num);
            }
        }
        result.to_string()
    }
}

// // 单元测试
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_nested_tags() {
//         let parser = MTag::new(
//             "嵌套测试: {$ToUpper({$GetFileName({@title/chinese}.txt)})".into()
//         );
//         assert_eq!(
//             parser.parse(),
//             "嵌套测试: 中文标题.TXT"
//         );
//     }

//     #[test]
//     fn test_math_operation() {
//         let parser = MTag::new(
//             "结果: {$Add(1, {$Multiply(2,3)})".into()
//         );
//         assert_eq!(
//             parser.parse(),
//             "结果: 7"
//         );
//     }
// }
