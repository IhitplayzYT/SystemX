pub mod command{
    use std::process::Command;
    use crate::tools::tools::Tools::{Tool,AgentContext};

    pub struct Bash;

    impl Tool for Bash{
        fn name(&self) -> &'static str {
            "Command"
        }

        fn description(&self) -> &'static str {
            "Executes a bash command only if user permits"
        }

        fn execute(
            &self,
            ctx:&mut AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let command_str = args.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;
            let args_vec: Vec<String> = command_str.split_whitespace().map(|s| s.to_string()).collect();
            let output = exec_cmnd(args_vec, &ctx.cwd);
            Ok(output)
        }
    }


    fn exec_cmnd(args: Vec<String>, cwd: &std::path::PathBuf) -> String{
        if args.len() == 0{
            return "No commands passed to execute".to_string();
        }
        let output = Command::new(&args[0]).args(&args[1..]).current_dir(cwd).stdin(std::process::Stdio::inherit()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).output();
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !stdout.is_empty() && !stderr.is_empty() {
                    format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr)
                } else if !stdout.is_empty() {
                    stdout
                } else if !stderr.is_empty() {
                    format!("STDERR:\n{}", stderr)
                } else {
                    format!("Command executed with exit code: {}", output.status.code().unwrap_or(-1))
                }
            }
            Err(e) => {
                format!("Failed to execute command: {}", e)
            }
        }
    }
    
}