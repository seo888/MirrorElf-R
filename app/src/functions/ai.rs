use crate::functions::func::MyFunc;
use std::{
    fs,
    // collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};
use super::{cerebras::CerebrasClient, html_to_yaml::HtmlYamlConverter, youdao::YouDao};
use crate::my_const::PROMPT_TRANS;

pub struct AiTrans {
    api_key: String,
    model: String,
    cerebras: CerebrasClient,
    my_func: Arc<MyFunc>,
}

impl AiTrans {
    pub fn new(api_key: &str, model: &str,my_func: Arc<MyFunc>) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            cerebras: CerebrasClient::new(api_key),
            my_func: my_func,
        }
    }

    pub async fn translate_html(
        &self,
        html_source: &str,
        from: &str,
        to: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let yaml = HtmlYamlConverter::html_to_yaml(&html_source, false)?;

        let prompt = PROMPT_TRANS.replace("{from}", from).replace("{to}", to);
        let message = format!("{}\n\n{}", prompt, yaml);
        // println!("Prompt: {}", message);

        // 写入临时YAML文件
        fs::write("output.yaml", &message)?;

        let new_yaml = match self.cerebras.chat_completion(&self.model, &message).await {
            Ok(response) => {
                let result = response.replace("```yml", "").replace("```", "");
                let result = result
                    .split_once("html:")
                    .map_or(result.as_str(), |(_prefix, content)| content)
                    .to_string();
                // println!("Result: {}", result);
                format!("html:{}", result)
            }
            Err(e) => {
                println!("Error: {}", e);
                return Err(e.into());
            }
        };

        let new_source = HtmlYamlConverter::replace_with_yaml(&html_source, &new_yaml)?;

        Ok(new_source)
    }

    pub async fn yd_translate_html(
        &self,
        html_source: &str,
        from: &str,
        to: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let yaml = HtmlYamlConverter::html_to_yaml(html_source, false)?;

        // let prompt = PROMPT_TRANS.replace("{from}", from)
        //     .replace("{to}", to);
        // let message = format!("{}\n\n{}", prompt, yaml);
        // println!("yaml: {}", yaml);

        // 写入临时YAML文件
        // fs::write("output.yaml", &message)?;
        let yd = YouDao {};
        // let result = yd.trans_yaml(&yaml, "en", "zh", "0.0.0.0").await.unwrap();

        // let new_yaml = match yd.trans_yaml(&yaml, from, to, "0.0.0.0").await {
        //     Ok(response) => {
        //         response
        //     },
        //     Err(e) => {
        //         println!("Error: {}", e);
        //         return Err(e.into());
        //     },
        // };

        // println!("new_yaml: {}", new_yaml);

        // let new_source = HtmlYamlConverter::replace_with_yaml(html_source, &new_yaml)?;

        let use_ip = &self.my_func.get_random_element(&self.my_func.ips);

        let new_source = match yd.trans_html(html_source, &yaml, from, to, use_ip).await {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                return Err(e.into());
            }
        };

        Ok(new_source)
    }
}
