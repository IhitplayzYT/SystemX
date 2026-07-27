pub mod html{
    use anyhow::anyhow;
use reqwest::blocking::Client;
    use scraper::{Html, Selector};
use serde::Deserialize;

use crate::tools::tools::Tools::Tool;

    fn get_url(url: &str) -> String{
        let html = reqwest::blocking::get(url).unwrap().text().unwrap();
        html
    }


    fn get_url_selector(url: &str, selectors: Vec<String>) -> String {
        let html = Client::new().get(url).send().map_err(|e| e.to_string()).unwrap().error_for_status().map_err(|e| e.to_string()).unwrap().text().map_err(|e| e.to_string()).unwrap();
        let page = Html::parse_document(&html);
        let mut ret = String::new();
        for j in &selectors {
            let selector = Selector::parse(j).map_err(|e| e.to_string()).unwrap();
            ret = ret + "\n\n" + j + ":\n";
            for i in page.select(&selector) {
                let text = i.text().collect::<String>().trim().to_owned();
                if !text.is_empty() {
                    ret = ret + &text + "\n";
                }
            }
        }
        ret
    }

    pub struct Get_html;
    impl Tool for Get_html{
        fn description(&self) -> &'static str {
            "Returns raw html for a url"
        }

        fn name(&self) -> &'static str {
            "get_html"
        } 

        fn execute(
            &self,
            _ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            
            let url = args.get("url").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing url"))?;
            Ok(get_url(url))
        }


    }

    pub struct Get_html_selectors;

#[derive(Debug, Deserialize)]
struct Selectors{
    selectors: Vec<String>,
}

    impl Tool for Get_html_selectors{
        fn description(&self) -> &'static str {
            "Returns String containg all elements of the corresponding provided array of html selectors"
        }

        fn name(&self) -> &'static str {
            "get_html_selectors"
        } 

        fn execute(
            &self,
            _ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            
            let url = args.get("url").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing url"))?;
            let selectors: Vec<String> = serde_json::from_value(args.get("selectors").ok_or_else(|| anyhow!("No selectors provided"))?.clone(),)?;
            Ok(get_url_selector(url,selectors))
        }


    }

    pub fn parse_html(body: &str,selectors: Vec<String>) -> String{
        let page = Html::parse_document(&body);
        let mut ret = String::new();
        for j in &selectors {
            let selector = Selector::parse(j).map_err(|e| e.to_string()).unwrap();
            ret = ret + "\n\n" + j + ":\n";
            for i in page.select(&selector) {
                let text = i.text().collect::<String>().trim().to_owned();
                if !text.is_empty() {
                    ret = ret + &text + "\n";
                }
            }
        }
        ret
    }

    pub struct Parse_html;

    impl Tool for Parse_html{
        fn name(&self) -> &'static str {
            "parse_html"
        }

        fn description(&self) -> &'static str {
            "Will take a html and parse elements of the required selectors provided in selectors array"
        }

        fn execute(
            &self,
            _ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            
            let body = args.get("body").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing html body"))?;
            let selectors: Vec<String> = serde_json::from_value(args.get("selectors").ok_or_else(|| anyhow!("No selectors provided"))?.clone(),)?;
            Ok(parse_html(body,selectors))

        }



    }







}