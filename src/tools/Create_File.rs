pub mod create_file{
    use std::{fs, path::PathBuf};

use anyhow::anyhow;

use path_clean::clean;
use crate::tool::{Create_Dir::create_dir::create_dir, tools::Tools::{AgentContext, Tool}};

    // Creates the dir and file if not exists and writes content to the file can be used for rewritting a file as well[Conditions managed internally]
    pub fn create_file(cwd:&PathBuf,ws:&PathBuf,target:&str,content:&str) -> String{
    let target_path = clean(cwd.join(PathBuf::from(&target)));

       if !target_path.starts_with(&cwd){
            return format!("Path {} does not Exist!",&target);
        }
        if !target_path.exists(){
            let k = create_dir(cwd, ws,&target[..]);
            if !k.starts_with("Created dir"){
                return k;
            }
        }
        if target_path.is_dir(){
          return format!("{} is not a File!",target);
        }
        let k = match fs::write(target_path, content){
            Err(_) => "Dir path created but something went wrong",
            _ => ""
        };
        if !k.is_empty(){
            return k.to_string();
        }
        format!("File {} created and written to",target)
    }

    pub struct touch;

    impl Tool for touch{

        fn name(&self) -> &'static str {
            "create_file"
        }
        fn description(&self) -> &'static str {
            "Creates the file with file contents provided alongside creation of intermediary directories in the path,"
        }
        fn execute(
        &self,
        ctx:&mut AgentContext,
        args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
            let content = args.get("content").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing content"))?;
            Ok(create_file(&ctx.cwd, &ctx.workspace,tgt,content))
        }

    }





}