pub mod cat_file{
    use std::{fs, path::PathBuf};


    use anyhow::anyhow;
    use crate::tools::tools::Tools::{AgentContext, Tool};

    pub fn cat_file(cwd:&PathBuf,ws: &PathBuf,target:&str) -> String{
        let target_path = cwd.join(PathBuf::from(&target)).canonicalize().unwrap();
        if !target_path.starts_with(ws) || !target_path.exists(){
            return format!("Path {} does not Exist!",&target);
        }
        if target_path.is_dir(){
            return format!("Path {} is not a file",&target);
        }
        fs::read_to_string(target_path).unwrap()
    }

    pub struct cat;

    impl Tool for cat{
        fn name(&self) -> &'static str {
            "cat_file"
        }
        fn description(&self) -> &'static str {
            "Used to cat the contents of file, will fail on directories,simila to linux 'cat' command"
        }
        fn execute(
            &self,
            ctx:&mut AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
            Ok(cat_file(&ctx.cwd, &ctx.workspace,tgt))
        }

    }


}


