pub mod excel_csv{

use calamine::{open_workbook_auto, Reader};

pub fn extract_csv(path: &str) -> String {
    match csv::Reader::from_path(path) {
        Ok(mut rdr) => {
            let mut out = String::new();
            for r in rdr.records() {
                match r {
                    Ok(record) => {
                        out = out + &record.iter().collect::<Vec<_>>().join(" ") + "\n";
                    }
                    Err(e) => {
                        return format!("Error reading CSV record: {}", e);
                    }
                }
            }
            out
        }
        Err(e) => format!("Error opening CSV file: {}", e)
    }
}


pub fn extract_excel(path: &str) -> String {
    match open_workbook_auto(path) {
        Ok(mut workbook) => {
            let mut out = String::new();
            for sheet in workbook.sheet_names().to_owned() {
                if let Ok(range) = workbook.worksheet_range(&sheet) {
                    out.push_str(&format!("Sheet: {}\n", sheet));
                    for row in range.rows() {
                        for cell in row {
                            out = out + &cell.to_string() + " ";
                        }
                        out += "\n";
                    }
                }
            }
            out
        }
        Err(e) => format!("Error opening Excel file: {}", e)
    }
}


}