pub mod docs{
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

pub fn extract_docx(path: &str) -> String {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return format!("Error opening file: {}", e),
    };
    
    let mut zip = match ZipArchive::new(file) {
        Ok(z) => z,
        Err(e) => return format!("Error opening zip archive: {}", e),
    };
    
    let mut xml = String::new();
    match zip.by_name("word/document.xml") {
        Ok(mut file) => {
            match file.read_to_string(&mut xml) {
                Ok(_) => {},
                Err(e) => return format!("Error reading XML: {}", e),
            }
        },
        Err(e) => return format!("Error finding document.xml: {}", e),
    }
    
    let mut reader = Reader::from_str(&xml);
    let mut out = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(e)) => {
                out = out + &String::from_utf8_lossy(e.as_ref()) + " ";
            }
            Ok(Event::Eof) => break,
            Err(e) => return format!("Error parsing XML: {}", e),
            _ => {}
        }
    }

    out
}

pub fn extract_doc(path: &str) -> String {
    use std::process::Command;
    
    // Try antiword first
    let output = Command::new("antiword")
        .arg(path)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).to_string()
            } else {
                // Try catdoc as fallback
                let catdoc_output = Command::new("catdoc")
                    .arg(path)
                    .output();
                
                match catdoc_output {
                    Ok(result) => {
                        if result.status.success() {
                            String::from_utf8_lossy(&result.stdout).to_string()
                        } else {
                            format!("Error: Both antiword and catdoc failed. Please install antiword or catdoc for .doc support. File: {}", path)
                        }
                    }
                    Err(_) => {
                        format!("Error: antiword/catdoc not found. Please install antiword or catdoc for .doc support. File: {}", path)
                    }
                }
            }
        }
        Err(_) => {
            // Try catdoc as fallback
            let catdoc_output = Command::new("catdoc")
                .arg(path)
                .output();
            
            match catdoc_output {
                Ok(result) => {
                    if result.status.success() {
                        String::from_utf8_lossy(&result.stdout).to_string()
                    } else {
                        format!("Error: Both antiword and catdoc failed. Please install antiword or catdoc for .doc support. File: {}", path)
                    }
                }
                Err(_) => {
                    format!("Error: antiword/catdoc not found. Please install antiword or catdoc for .doc support. File: {}", path)
                }
            }
        }
    }
}

pub fn extract_document(path: &str) -> String {
    if path.ends_with(".docx") {
        extract_docx(path)
    } else if path.ends_with(".doc") {
        extract_doc(path)
    } else {
        format!("Unsupported document format: {}", path)
    }
}

}