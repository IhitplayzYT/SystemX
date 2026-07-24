pub mod create_dir{
    use std::{fs, path::PathBuf};

use crate::tool::tools::Tools::{AgentContext, Tool};
use anyhow::anyhow;
use path_clean::clean;

    // Will create dir if file at the end of dir path then the file wont be created
    pub fn create_dir(cwd:&PathBuf,ws:&PathBuf,target:&str) -> String{
        let mut target_path = clean(cwd.join(PathBuf::from(&target)));
        println!("{:?} {:?}",cwd,target_path);
        if !target_path.starts_with(&ws){
            return format!("Path {} does not Exist!",target);
        }
        if target_path.exists(){
            return format!("Path {} already Exists!",target)
        }
        if target_path.is_file(){
            target_path = target_path.parent().unwrap().to_path_buf();
        }
        if target_path.iter().last().unwrap().to_str().unwrap().find(".").unwrap_or(0) > 0{
            target_path = target_path.parent().unwrap().to_path_buf();
        };
        let k = match fs::create_dir_all(&target_path){
            Err(_) => &format!("Failed to create dir: {}",target),
            _ => ""
        };
        if !k.is_empty(){
            return k.to_string();
        }
        format!("Created dir: {}",target)
    }

pub struct mkdir;


impl Tool for mkdir{
    fn name(&self) -> &'static str {
        "create_dir"
    }
    fn description(&self) -> &'static str {
        "Creates the directories provided in the path,i.e similar to mkdir -p ,if the last entry in path contains an extention(i.e is a file) then that file wont be created however rest of directories required would be created"
    }
    fn execute(
        &self,
        ctx:&mut AgentContext,
        args: serde_json::Value,
    ) -> anyhow::Result<String>
    {
        let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
        Ok(create_dir(&ctx.cwd,&ctx.workspace, tgt))
    }

}


}