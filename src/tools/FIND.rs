pub mod find{

use anyhow::{Result, anyhow, bail};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::tool::tools::Tools::{AgentContext, Tool};

pub fn find_file(
    cwd: &PathBuf,
    ws:&PathBuf,
    target: &str,
) -> Result<String> {

    let cwd = cwd.canonicalize()?;

    let mut out = String::new();

    for entry in WalkDir::new(&cwd)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !path.starts_with(ws){
            continue;
        }

        if entry.file_type().is_file() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains(target) {
                    let rel = path.strip_prefix(&cwd)?;
                    out.push_str(rel.to_string_lossy().as_ref());
                    out.push('\n');
                }
            }
        }
    }

    if out.is_empty() {
        bail!("No matching files found.");
    }

    Ok(out)
}

    pub struct find;

    impl Tool for find{

        fn name(&self) -> &'static str {
            "find"
        }
        fn description(&self) -> &'static str {
            "Recursivly searches for a target file starting from the cwd"
        }
        fn execute(
        &self,
        ctx: &mut AgentContext,
        args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("target").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing target"))?;
            Ok(find_file(&ctx.cwd,&ctx.workspace, tgt).unwrap_or("Find failed".to_string()))
        }

    }



}