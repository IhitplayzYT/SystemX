pub mod ingestor{
    use std::{fs, path::Path};

use crate::ingestors::{Docs::docs::extract_document, Excel_Csv::excel_csv::{extract_csv, extract_excel}, Html::html::get_url, Pdf::pdf::extract_pdf};



    pub async fn ingest(path:&str) -> String{
     let ext = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        match &ext[..]{
            "pdf" => extract_pdf(path),
            "docx" => extract_document(path),
            "xlsx" | "xls" => extract_excel(path),
            "csv" => extract_csv(path),
            _ => {

                if path.contains("http"){
                    get_url(path).await
                }else{
                fs::read_to_string(path).unwrap()
                }
            }
        }
    }


    
}