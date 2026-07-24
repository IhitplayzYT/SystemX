pub mod list_dir{
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

use anyhow::anyhow;

use crate::tool::tools::Tools::{AgentContext, Tool, mode_to_string};


    pub fn list_dir(cwd:&PathBuf,ws:&PathBuf,target:&str) -> String{
        let mut ret = String::new();
        let target_path = cwd.join(PathBuf::from(&target)).canonicalize().unwrap();
        if !target_path.starts_with(ws) || !target_path.exists(){
            return format!("Path {} does not Exist!\n",&target);
        }
        fs::read_dir(target_path).unwrap().for_each(|x|{
            let content = x.unwrap();
            let fullpath = content.path();
            let relative = fullpath.strip_prefix(&cwd).unwrap();
            let meta = content.metadata().unwrap();
            let perm = meta.permissions();
            ret += &format!("{} {} {}B\n",relative.to_str().unwrap(),mode_to_string(perm.mode()),meta.len())
        });
        ret            
    }
pub struct ls;

impl Tool for ls{
    fn name(&self) -> &'static str {
        "list_dir"
    }

    fn description(&self) -> &'static str {
        "List directory contents alongside some file metadata like permissionsand size"
    }

    fn execute(
        &self,
        ctx: &mut AgentContext,
        args: serde_json::Value,
    ) -> anyhow::Result<String>
    {
        let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
        Ok(list_dir(&ctx.cwd,&ctx.workspace, tgt))
    }

}

}