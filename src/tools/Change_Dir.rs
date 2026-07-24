pub mod change_dir{
    use std::path::PathBuf;

use anyhow::anyhow;
use path_clean::PathClean;

use crate::tool::tools::Tools::{AgentContext, Tool};


    pub fn change_dir(cwd:&mut PathBuf,ws:&PathBuf,target:&str) -> String{
        let mut target_path = cwd.join(PathBuf::from(&target)).canonicalize().unwrap();
        if !target_path.starts_with(&ws) || !target_path.exists(){
            return format!("Path {} does not Exist!",&target);
        }
        if target_path.is_file(){
            target_path = target_path.parent().ok_or("No parent dir").unwrap().to_path_buf();
        }

        
        match std::env::set_current_dir(target_path.clone()){
            Ok(_) => {*cwd = target_path.clone();format!("Changed working directory: {:?}",target_path)},
            _ => format!("Failed to change working directory")
        }
    }

    pub struct cd;

impl Tool for cd{
fn name(&self) -> &'static str {
    "change_dir"
}
fn description(&self) -> &'static str {
    "changes the cwd incase path is to a file it changes to the file's parent directory"
}
fn execute(
        &self,
        ctx: &mut AgentContext,
        args: serde_json::Value,
    ) -> anyhow::Result<String>
    {
        let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
        Ok(change_dir(&mut ctx.cwd,&ctx.workspace,tgt))
    }


}


}