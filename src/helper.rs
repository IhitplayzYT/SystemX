#[allow(dead_code,non_camel_case_types,non_snake_case)]

pub mod Helper {
    use std::{fs::{self, exists}, path::{Path, PathBuf}};

use crate::{chunker::chunker::Chunker, vstore::embed::embed::EmbedMethod};



    const DBG_STR: &str = "Usage";
    pub const OK: i32 = 0;
    pub const ERR: i32 = 1;
    pub const END_POINT:&str = "http://localhost:11434";
    pub const MODEL:&str = "llama3.2";
    pub const VSTORE_N: &str = "./data/vstore";
    pub const COLLECTION_N: &str = "document_chunks";
 

    #[derive(Debug,Clone)]
    pub struct CLI {
        pub debug: bool,
        pub srcdir: Vec<String>,
        pub srcfile: Vec<String>,
        pub chunker:Chunker,
        pub query: String,
        pub context_file: Option<String>,
        pub model:String,
        pub temp: f32,
        pub vdim: usize,
        pub vstore:Option<String>,
        pub collection: Option<String>,
        pub embed_model:Option<EmbedMethod>,
        pub window_len: Option<usize>
        pub url: String,
        pub token_limits:(usize,usize,usize), // (min_context,max_context,max_output)
        pub root_dir: PathBuf,
        pub steps:usize,
        pub memory:Option<String>,
        pub sprompt:Option<String>,
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
                temp: 0.4,
                vdim: 384,
                vstore: None,
                collection: None,
                embed_model: None,
                window_len: None
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
                } else if i.starts_with("--vdim=") || i.starts_with("-n="){
                    self.vdim = i[i.find("=").unwrap()+1..].trim().parse::<usize>().expect("Vdim is a usize which decides dimensions of embeddings");
                } else if i.starts_with("--vstore=") || i.starts_with("-v="){
                    self.vstore = Some(i[i.find("=").unwrap()+1..].trim().to_string());
                } else if i.starts_with("--collection=") || i.starts_with("-c="){
                    self.collection = Some(i[i.find("=").unwrap()+1..].trim().to_string());
                } else if i.starts_with("--embed-model=") || i.starts_with("--embedder=") || i.starts_with("--embed=")|| i.starts_with("-e="){
                    self.embed_model = Some(EmbedMethod::from(i[i.find("=").unwrap()+1..].trim().to_string()));
                } else if i.starts_with("--winlen=") || i.starts_with("--win-len=") || i.starts_with("--win=") || i.starts_with("-w="){
                    self.window_len = Some(i[i.find("=").unwrap()+1..].trim().parse::<usize>().expect("Win Len is a optional param that can be provided when using cluster embedder"));
                } else if i.starts_with("--url=") || i.starts_with("-u="){
                    self.url = i[i.find("=").unwrap()+1..].trim().to_string();
                } else if i.starts_with("--min="){
                    self.token_limits.0 = i[i.find("=").unwrap()+1..].trim().parse::<usize>().expect("Min Tokens is a non negative usize"); 
                } else if i.starts_with("--max="){
                    self.token_limits.1 = i[i.find("=").unwrap()+1..].trim().parse::<usize>().expect("Max Tokens is a non negative usize,for unbounded limit use --max=0"); 
                    if self.token_limits.1 == 0{
                        self.token_limits.1 = usize::MAX;
                    }
                } else if i.starts_with("--maxout="){
                    self.token_limits.2 = i[i.find("=").unwrap()+1..].trim().parse::<usize>().expect("Max Output Tokens is a non negative usize"); 
                }
                 else if i.starts_with("--root=") || i.starts_with("--idir"){
                    self.root_dir= PathBuf::from(&i.split_off(i.find("=").unwrap())[1..]).canonicalize().unwrap();
                }else if i.starts_with("--steps=") || i.starts_with("-s="){ 
                    self.steps = (i.split_off(i.find("=").unwrap()+1)).parse().expect("Steps has to be usize");
                }else if i.starts_with("--memory="){ 
                    self.memory = Some(i.split_off(i.find("=").unwrap()+1));
                }else if i.starts_with("--sysprompt=") || i.starts_with("--prompt="){ 
                    self.sprompt = Some(i.split_off(i.find("=").unwrap() + 1));
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
