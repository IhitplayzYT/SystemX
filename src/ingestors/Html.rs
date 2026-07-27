pub mod html{
use scraper::{Html, Selector};



    pub async fn get_url(url: &str) -> String{
        let html = reqwest::get(url).await.map_err(|x| format!("Failed due to: {x}")).unwrap().text().await.map_err(|x| format!("Failed due to: {x}")).unwrap();
            html
    }

    pub async fn get_url_selectors(url: &str, selectors: Vec<&str>) -> String { 
        let html = reqwest::get(url).await.map_err(|x| format!("Failed due to: {x}")).unwrap().text().await.map_err(|x| format!("Failed due to: {x}")).unwrap();
        let page = Html::parse_document(&html);
        let mut ret = String::new();
        for j in selectors {
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

    



}