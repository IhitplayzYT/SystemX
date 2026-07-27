pub mod pdf{

    pub fn extract_pdf(path: &str) -> String {
        pdf_extract::extract_text(path).map_err(|e| e.to_string()).unwrap()
    }


}