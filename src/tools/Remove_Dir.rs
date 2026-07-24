pub mod remove_dir{
    use std::{fs, path::PathBuf};

use anyhow::anyhow;

use crate::tool::tools::Tools::{AgentContext, Tool};



    // Can delete files as well
    pub fn remove_dir(cwd:&PathBuf,ws:&PathBuf, target: &str) -> String {
        // Resolve relative to the sandbox root, not the process cwd.
        let joined = cwd.join(&target);
        let target_path = match joined.canonicalize() {
            Ok(p) => p,
            Err(_) => return format!("Path '{}' does not Exist!", target),
        };
        if !target_path.starts_with(ws) {
            return format!("Path {} does not Exist!",&target);

        }
        let result = if target_path.is_dir() {
            fs::remove_dir_all(&target_path)
        } else {
            fs::remove_file(&target_path)
        };
        match result {
            Ok(_) => format!("Removed '{}'", target),
            Err(e) => format!("Failed to remove '{}': {}", target, e),
        }
    }

    pub struct rm;

    impl Tool for rm{
        fn name(&self) -> &'static str {
            "remove_dir"
        }

        fn description(&self) -> &'static str {
            "Similar to Linux rm -rf used to delete files and directories"
        }
        fn execute(
        &self,
        ctx:&mut AgentContext,
        args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
            Ok(remove_dir(&ctx.cwd,&ctx.workspace, tgt))
        }
        

    }



}