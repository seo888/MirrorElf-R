// mod functions;
// use functions::cerebras::CerebrasClient;
use html_escape::{decode_html_entities, encode_text};
use indexmap::IndexMap;
use regex::Regex;
use scraper::{node::Node, ElementRef, Html, Selector};
use serde::{Serialize, Serializer};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::fs;

// 定义YamlNode结构体，用于构建YAML树结构
#[derive(Debug, Default)]
struct YamlNode {
    children: IndexMap<String, YamlNode>, // 子节点
    text: Option<String>,                 // 节点文本内容
}

impl YamlNode {
    // 判断节点是否为叶子节点（无子节点）
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

// HtmlYamlConverter实现
pub struct HtmlYamlConverter {}

impl HtmlYamlConverter {
    /// 将HTML内容转换为YAML字符串
    ///
    /// 参数：
    /// - html: 输入HTML字符串
    /// - length_mode: 是否以文本长度代替文本内容
    ///
    /// 返回：
    /// - Result<String>: 转换后的YAML字符串，错误时返回错误信息
    pub fn html_to_yaml(html: &str, length_mode: bool) -> Result<String, String> {
        let tree = Self::build_yaml_tree(&html.replace("&nbsp;", " ").replace(" ", " "))?;
        let yaml_value = Self::node_to_yaml(&tree, length_mode);
        let cleaned_value = Self::remove_empty_values(&yaml_value);
        serde_yaml::to_string(&cleaned_value).map_err(|e| format!("YAML序列化失败: {}", e))
        // .map(|s| s.replace("'''", "'"))
    }

    /// 根据YAML替换HTML内容
    ///
    /// 参数：
    /// - html: 输入HTML字符串
    /// - yaml: 替换内容的YAML字符串
    ///
    /// 返回：
    /// - Result<String>: 替换后的HTML字符串，错误时返回错误信息
    pub fn replace_with_yaml(html: &str, yaml: &str) -> Result<String, String> {
        let value: Value =
            serde_yaml::from_str(yaml).map_err(|e| format!("YAML解析失败: {}", e))?;

        let doc = Html::parse_document(&html);
        let selector = Selector::parse("*").unwrap();
        let mut modified = doc.root_element().html().replace("&nbsp;", " ").replace(" ", " ");

        // 建立XPath到元素的映射
        let mut xpath_to_element = HashMap::new();
        for element in doc.select(&selector) {
            let xpath = Self::generate_xpath(&element);
            xpath_to_element.insert(xpath, element);
        }

        Self::process_yaml_value(&xpath_to_element, "", &value, &mut modified)?;
        Ok(modified)
    }

    /// 构建YAML树结构
    ///
    /// 参数：
    /// - html: 输入HTML字符串
    ///
    /// 返回：
    /// - Result<YamlNode>: 构建的YAML树，错误时返回错误信息
    fn build_yaml_tree(html: &str) -> Result<YamlNode, String> {
        let doc = Html::parse_document(html);
        let selector = Selector::parse("*").map_err(|e| e.to_string())?;
        let skip_tags = ["script", "style", "noscript", "meta", "link", "time"];
        // let skip_tags = ["script", "style", "noscript", "meta", "link", "time", "a"];

        let mut root = YamlNode::default();

        for element in doc.select(&selector) {
            let tag = element.value().name();
            if skip_tags.contains(&tag) {
                continue;
            }

            let text = Self::extract_direct_text(&element);

            let mut current = &mut root;
            let path = Self::build_xpath_stack(&element);

            for (index, component) in path.iter().enumerate() {
                if index == path.len() - 1 {
                    if !text.is_empty() {
                        let key = component.clone();
                        current.children.entry(key).or_default().text = Some(text.clone());
                    }
                } else {
                    current = current.children.entry(component.clone()).or_default();
                }
            }
        }

        Ok(root)
    }

    fn normalize_whitespace(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut in_whitespace = false;

        for c in s.chars() {
            if c.is_whitespace() {
                if !in_whitespace {
                    result.push(' ');
                    in_whitespace = true;
                }
            } else {
                result.push(c);
                in_whitespace = false;
            }
        }
        result
    }

    /// 提取元素的直接文本内容
    ///
    /// 参数：
    /// - element: 输入元素引用
    ///
    /// 返回：
    /// - String: 提取的文本内容
    fn extract_direct_text(element: &ElementRef) -> String {
        element
            .children()
            .filter_map(|node| {
                if let Node::Text(text) = node.value() {
                    Some(Self::normalize_whitespace(&text.trim()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// 将YamlNode转换为serde_yaml::Value
    ///
    /// 参数：
    /// - node: YamlNode引用
    /// - length_mode: 是否以文本长度代替文本内容
    ///
    /// 返回：
    /// - Value: 转换后的YAML值
    ///

    fn node_to_yaml(node: &YamlNode, length_mode: bool) -> Value {
        if node.is_leaf() {
            node.text.as_ref().map_or(Value::Null, |t| {
                // 只在文本长度>1时包含
                if t.chars().count() > 1 {
                    if length_mode {
                        Value::Number(t.chars().count().into())
                    } else {
                        // 用单引号包裹字符串
                        Value::String(format!("'{}'", t))
                    }
                } else {
                    Value::Null
                }
            })
        } else {
            let mut map = Mapping::new();
            let mut has_valid_content = false;

            if let Some(text) = &node.text {
                let char_count = text.chars().count();
                // 检查文本长度是否>1
                if !text.is_empty() && char_count > 1 {
                    let value = if length_mode {
                        Value::Number(char_count.into())
                    } else {
                        // 用单引号包裹字符串
                        Value::String(format!("'{}'", text))
                    };
                    map.insert(Value::String("~".to_string()), value);
                    has_valid_content = true;
                }
            }

            for (k, v) in &node.children {
                let child_value = Self::node_to_yaml(v, length_mode);
                if !Self::is_empty_value(&child_value) {
                    map.insert(Value::String(k.clone()), child_value);
                    has_valid_content = true;
                }
            }

            if has_valid_content {
                if map.len() == 1 && map.contains_key(&Value::String("~".to_string())) {
                    map.get(&Value::String("~".to_string()))
                        .cloned()
                        .unwrap_or(Value::Null)
                } else {
                    Value::Mapping(map)
                }
            } else {
                Value::Null
            }
        }
    }

    /// 判断YAML值是否为空
    ///
    /// 参数：
    /// - value: YAML值
    ///
    /// 返回：
    /// - bool: 是否为空
    fn is_empty_value(value: &Value) -> bool {
        matches!(value, Value::Null) || matches!(value, Value::Mapping(m) if m.is_empty())
    }

    /// 移除空值
    ///
    /// 参数：
    /// - value: YAML值
    ///
    /// 返回：
    /// - Value: 移除空值后的YAML值
    fn remove_empty_values(value: &Value) -> Value {
        match value {
            Value::Mapping(m) => {
                let mut cleaned = Mapping::new();
                for (k, v) in m {
                    let cleaned_v = Self::remove_empty_values(v);
                    if !Self::is_empty_value(&cleaned_v) {
                        cleaned.insert(k.clone(), cleaned_v);
                    }
                }
                if cleaned.is_empty() {
                    Value::Null
                } else {
                    Value::Mapping(cleaned)
                }
            }
            Value::Sequence(s) => {
                let cleaned: Vec<_> = s
                    .iter()
                    .map(Self::remove_empty_values)
                    .filter(|v| !Self::is_empty_value(v))
                    .collect();
                Value::Sequence(cleaned)
            }
            _ => value.clone(),
        }
    }

    /// 处理YAML值并将其应用到HTML中
    ///
    /// 参数：
    /// - xpath_to_element: XPath到元素的映射
    /// - current_path: 当前XPath路径
    /// - value: YAML值
    /// - modified: 修改后的HTML字符串
    ///
    /// 返回：
    /// - Result<(), String>: 错误信息
    fn process_yaml_value(
        xpath_to_element: &HashMap<String, ElementRef>,
        current_path: &str,
        value: &Value,
        modified: &mut String,
    ) -> Result<(), String> {
        match value {
            Value::Number(n) => {
                Self::apply_text_replace(xpath_to_element, current_path, n, modified)
            }
            Value::Mapping(map) => {
                if let Some(text_value) = map.get(&Value::String("~".into())) {
                    Self::process_tilde_node(xpath_to_element, current_path, text_value, modified)?;
                }

                for (k, v) in map {
                    if k != "~" {
                        let key = k.as_str().ok_or("非字符串键")?;
                        let new_path = format!("{}/{}", current_path, key);
                        Self::process_yaml_value(xpath_to_element, &new_path, v, modified)?;
                    }
                }
                Ok(())
            }
            Value::String(s) => {
                Self::apply_text_replace(xpath_to_element, current_path, s, modified)
            }
            _ => Err(format!("不支持的YAML类型: {:?}", value)),
        }
    }

    /// 处理~节点（直接文本节点）
    ///
    /// 参数：
    /// - xpath_to_element: XPath到元素的映射
    /// - base_path: 基础XPath路径
    /// - value: YAML值
    /// - modified: 修改后的HTML字符串
    ///
    /// 返回：
    /// - Result<(), String>: 错误信息
    fn process_tilde_node(
        xpath_to_element: &HashMap<String, ElementRef>,
        base_path: &str,
        value: &Value,
        modified: &mut String,
    ) -> Result<(), String> {
        match value {
            Value::Number(n) => Self::apply_text_replace(xpath_to_element, base_path, n, modified),
            Value::String(s) => Self::apply_text_replace(xpath_to_element, base_path, s, modified),
            _ => Err("~节点值必须是数字或字符串".into()),
        }
    }

    /// 应用文本替换
    ///
    /// 参数：
    /// - xpath_to_element: XPath到元素的映射
    /// - path: XPath路径
    /// - value: 要替换的值
    /// - modified: 修改后的HTML字符串
    ///
    /// 返回：
    /// - Result<(), String>: 错误信息
    fn apply_text_replace<T: ToString>(
        xpath_to_element: &HashMap<String, ElementRef>,
        path: &str,
        value: T,
        modified: &mut String,
    ) -> Result<(), String> {
        let text = value.to_string();
        if let Some(elem) = xpath_to_element.get(path) {
            Self::replace_element_text(*elem, &text, modified);
        } else {
            return Err(format!("未找到元素: path = {}", path));
        }
        Ok(())
    }

    /// 替换元素的文本内容
    ///
    /// 参数：
    /// - element: 元素引用
    /// - new_text: 新的文本内容
    /// - html: 修改后的HTML字符串
    fn replace_element_text(element: ElementRef, new_text: &str, html: &mut String) {
        if let Some(node) = element.first_child() {
            if let Some(text_node) = node.value().as_text() {
                let original_text = text_node.text.as_ref();
                let escaped_text = encode_text(new_text);
                // 解码HTML实体后再查找位置
                let decoded_text = decode_html_entities(original_text).into_owned();
                if let Some(range) = Self::find_text_range(html, &decoded_text, &element) {
                    // 将新文本进行HTML转义
                    html.replace_range(range, &escaped_text);
                }
            }
        }
    }

    // /// 查找文本范围
    // ///
    // /// 参数：
    // /// - html: HTML字符串
    // /// - text: 要查找的文本
    // /// - element: 元素引用
    // ///
    // /// 返回：
    // /// - Option<std::ops::Range<usize>>: 文本范围
    fn find_text_range(
        html: &str,
        text: &str,
        element: &ElementRef,
    ) -> Option<std::ops::Range<usize>> {
        let element_html = element.html();
        let element_start_in_html = html.rfind(&element_html)?;

        // 在原始编码文本中查找
        let encoded_text = encode_text(text).to_string();
        let text_start_in_element = element_html.rfind(&encoded_text)?;

        let absolute_start = element_start_in_html + text_start_in_element;
        let absolute_end = absolute_start + encoded_text.len(); // 使用编码后长度

        if absolute_end > html.len() {
            None
        } else {
            Some(absolute_start..absolute_end)
        }
    }

    /// 生成XPath表达式
    ///
    /// 参数：
    /// - element: 元素引用
    ///
    /// 返回：
    /// - String: 生成的XPath表达式
    fn generate_xpath(element: &ElementRef) -> String {
        let mut components = Vec::new();
        let mut current = Some(*element);

        while let Some(elem) = current {
            let tag = elem.value().name();
            let parent = elem.parent().and_then(ElementRef::wrap);

            let index = parent.and_then(|p| Self::count_siblings(p, elem, tag));

            let component = match index {
                Some(i) if i > 1 => format!("{}[{}]", tag, i),
                Some(1) => tag.to_string(),
                _ => tag.to_string(),
            };

            components.push(component);
            current = parent;
        }

        components.reverse();
        format!("/{}", components.join("/"))
    }

    /// 计数同名的兄弟元素
    ///
    /// 参数：
    /// - parent: 父亲元素
    /// - target: 目标元素
    /// - tag: 标签名
    ///
    /// 返回：
    /// - Option<usize>: 位置索引
    fn count_siblings(parent: ElementRef, target: ElementRef, tag: &str) -> Option<usize> {
        let mut count = 0;
        let mut position = 0;

        for child in parent.children().filter_map(ElementRef::wrap) {
            if child.value().name() == tag {
                count += 1;
                if child.id() == target.id() {
                    position = count;
                }
            }
        }

        if count > 1 { Some(position) } else { None }
    }

    /// 构建XPath栈
    ///
    /// 参数：
    /// - element: 元素引用
    ///
    /// 返回：
    /// - Vec<String>: XPath栈
    fn build_xpath_stack(element: &ElementRef) -> Vec<String> {
        let mut stack = Vec::new();
        let mut current = Some(*element);

        while let Some(elem) = current {
            let tag = elem.value().name();
            let parent = elem.parent().and_then(ElementRef::wrap);

            let index = parent.and_then(|p| Self::count_siblings(p, elem, tag));

            let component = match index {
                Some(i) if i > 1 => format!("{}[{}]", tag, i),
                Some(1) => tag.to_string(),
                _ => tag.to_string(),
            };

            stack.push(component);
            current = parent;
        }

        stack.reverse();
        stack
    }
}