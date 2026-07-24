pub mod cargo_call{
    use std::{path::PathBuf, process::Command};

use anyhow::anyhow;

use crate::tool::tools::Tools::{AgentContext, Tool};


pub fn cargo_call(cwd:&PathBuf,ws:&PathBuf,target: &str, args: &Vec<String>) -> String {
    let joined = cwd.join(&target);
    let target_path = match joined.canonicalize() {
        Ok(p) => p,
        Err(_) => return format!("Path '{}' does not exist!", target),
    };
    if !target_path.starts_with(ws) {
        return format!("Access denied: '{}' is outside the working directory.", target);
    }
    if !target_path.is_dir() {
        return format!("'{}' is not a directory.", target);
    }
    if !target_path.join("Cargo.toml").exists() {
        return format!("'{}' is not a Cargo project.", target);
    }

    let output = match Command::new("cargo")
        .args(args)
        .current_dir(&target_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => return format!("Failed to execute cargo: {}", e),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        if stdout.trim().is_empty() {
            format!("Cargo command completed successfully.")
        } else {
            stdout.into_owned()
        }
    } else {
        format!(
            "Cargo command failed (exit code {:?})\n\nstdout:\n{}\n\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        )
    }
}

pub struct cargo;

impl Tool for cargo{
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn description(&self) -> &'static str {
        "Calls cargo with params provided on the target path"
    }

    fn execute(
        &self,
        ctx: &mut AgentContext,
        args: serde_json::Value,
    ) -> anyhow::Result<String>
    {
        let tgt = args.get("path").and_then(serde_json::Value::as_str).ok_or_else(|| anyhow!("Missing path"))?;
    let params: Vec<String> = serde_json::from_value(
        args.get("params")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing params"))?,
    )?;
        anyhow::Ok(cargo_call(&ctx.cwd,&ctx.workspace,tgt, &params))
        
    }

}




}