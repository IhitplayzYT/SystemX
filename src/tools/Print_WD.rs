pub mod print_wd{
    use std::path::PathBuf;

use crate::tool::tools::Tools::{AgentContext, Tool};


pub fn print_wd(cwd:&PathBuf) -> String{
    match std::env::current_dir().unwrap().strip_prefix(&cwd){
        Ok(x) => format!("Current WD: {:?}",x),
        _ => format!("CRITICAL Error!Abort!")
    }
}

pub struct pwd;

impl Tool for pwd{


    fn name(&self) -> &'static str {
        "print_wd"
    }
    fn description(&self) -> &'static str {
        "Prints the current working directory"
    }
    fn execute(
        &self,
        ctx:&mut AgentContext,
        _args: serde_json::Value,
    ) -> anyhow::Result<String>
    {
       anyhow::Ok(print_wd(&ctx.cwd)) 
    }

}




}