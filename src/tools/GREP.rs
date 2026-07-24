pub mod grep{

use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::tool::tools::Tools::{AgentContext, Tool};

pub fn grep(
    cwd: &PathBuf,
    ws:&PathBuf,
    needle: &str,
) -> Result<String> {

    let cwd = cwd.canonicalize()?;

    let mut out = String::new();

    for entry in WalkDir::new(&cwd)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if !path.starts_with(ws){
            continue;
        }

        let Ok(contents) = fs::read_to_string(path) else {
            // skip binary files
            continue;
        };

        for (line_no, line) in contents.lines().enumerate() {

            if line.contains(needle) {

                let rel = path.strip_prefix(&cwd)?;

                out.push_str(&format!(
                    "{}:{}:{}\n",
                    rel.display(),
                    line_no + 1,
                    line
                ));
            }
        }
    }

    if out.is_empty() {
        out.push_str("No matches found.");
    }

    Ok(out)
}

    pub struct Grep;

    impl Tool for Grep{

        fn name(&self) -> &'static str {
            "grep"
        }
        fn description(&self) -> &'static str {
            "Recursivly searches for a target string starting from the cwd"
        }
        fn execute(
        &self,
        ctx: &mut AgentContext,
        args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let tgt = args.get("target").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing target"))?;
            Ok(grep(&ctx.cwd,&ctx.workspace, tgt).unwrap_or("Grep failed".to_string()))
        }

    }




}