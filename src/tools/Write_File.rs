pub mod write_file{
    use std::{fs::write, path::PathBuf};

use crate::tool::{Create_Dir::create_dir::create_dir, tools::Tools::{AgentContext, Tool}};

use anyhow::anyhow;
use path_clean::clean; 
    pub fn _write_file(cwd:&PathBuf,ws:&PathBuf,target: &str,content:&str) -> String{
    let target_path = clean(cwd.join(PathBuf::from(&target)));

       if !target_path.starts_with(ws){
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
        
        write(target_path, content).unwrap();
        format!("File {} written to",target)
    }

    pub struct write_file;

    impl Tool for write_file{

        fn name(&self) -> &'static str {
            "write_file"
        }
        fn description(&self) -> &'static str {
            "Creates the file with file contents provided alongside creation of intermediary directories in the path,simiar to tool create_file"
        }
        fn execute(
        &self,
        ctx:&mut AgentContext,
        args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
            let content = args.get("content").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing content"))?;
            Ok(_write_file(&ctx.cwd,&ctx.workspace, tgt,content))
        }

    }

}