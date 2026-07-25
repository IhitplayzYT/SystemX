#[allow(dead_code,non_camel_case_types,non_snake_case)]

pub mod Helper {
    use std::{fs::{self, exists}, path::Path};

use crate::chunker::chunker::Chunker;



    const DBG_STR: &str = "Usage";
    pub const OK: i32 = 0;
    pub const ERR: i32 = 1;
    pub const END_POINT:&str = "http://localhost:11434";
    pub const MODEL:&str = "llama3.2";
 

    #[derive(Debug,Clone)]
    pub struct CLI {
        pub debug: bool,
        pub srcdir: Vec<String>,
        pub srcfile: Vec<String>,
        pub chunker:Chunker,
        pub query: String,
        pub context_file: Option<String>,
        pub model:String,
        pub temp: f32
        
    }

    impl CLI {
        pub fn new() -> CLI {
            Self {
                debug: false,
                srcdir: vec![],
                chunker:Chunker::LINE,
                srcfile:vec![],
                query: String::new(),
                context_file: None,
                model: "llama3.2".to_string(),
                temp: 0.4
            }
        }

        pub fn Parse_Args(&mut self) {
            let clargs = std::env::args().collect::<Vec<String>>();

            if clargs.is_empty() {
                return;
            }
            for i in clargs.iter().skip(1).collect::<Vec<&String>>() {
                
                if i == "-h" || i == "--help" || i == "-H" || i == "--Help" {
                    Help();
                } else if i == "-d" || i == "--debug" || i == "-D" || i == "--Debug" {
                    self.debug = true
                } else if i.starts_with("--srcdir=") || i.starts_with("--SRC_DIR=") {
                    let idx = i.find("=").unwrap();
                    self.srcdir.push(i[idx + 1..].to_string());
                }  else if i.starts_with("--srcfile=") || i.starts_with("--SRC_FILE=") {
                    let idx = i.find("=").unwrap();
                    self.srcfile.push(i[idx + 1..].to_string());
                } else if i.starts_with("--chunker=") || i.starts_with("-s="){
                    let idx = i.find("=").unwrap();
                    self.chunker = Chunker::from(i[idx + 1..].trim().to_string());
                } else if i.starts_with("--query=") || i.starts_with("-q="){
                    self.query = i[i.find("=").unwrap()+1..].trim().to_string();
                } else if i.starts_with("--context=") || i.starts_with("-c="){
                    self.context_file = Some(i[i.find("=").unwrap()+1..].trim().to_string());
                } else if i.starts_with("--model=") || i.starts_with("-m="){
                    self.model = i[i.find("=").unwrap()+1..].trim().to_string();
                } else if i.starts_with("--temp=") || i.starts_with("-t="){
                    self.temp = i[i.find("=").unwrap()+1..].trim().parse::<f32>().expect("Temp is a float between 0.0 - 2.0,decides creativity of model");
                }
                 else {
                    Help();
                }
            }
            println!("{:?}",self);
        }
    }

    pub fn Help() {
        println!("{DBG_STR}");
        std::process::exit(OK);
    }

    pub fn unwrap_dirs(mut dirs:Vec<String>,ret: &mut Vec<String>){
        for i in &dirs.clone(){
            let x = Path::new(i);
            if x.exists(){
                if x.is_file(){
                    ret.push(i.to_string());
                }else if x.is_dir(){
                    dirs.append(&mut fs::read_dir(x).unwrap().into_iter().map(|m| {let n = m.unwrap();return n.path().to_str().unwrap().to_string()}).collect::<Vec<String>>());
                    unwrap_dirs(dirs, ret);
                    return;
                }
            }
            
        }

    }

}
