pub mod Tools{
    use std::{collections::HashMap, path::PathBuf};

use crate::tool::{Cargo_Call::cargo_call::cargo, Cat_File::cat_file::cat, Change_Dir::change_dir::cd, Create_Dir::create_dir::mkdir, Create_File::create_file::touch, FIND::find::find, GREP::grep::Grep, List_Dir::list_dir::ls, Modify_File::modify_file::edit, Print_WD::print_wd::pwd, Remove_Dir::remove_dir::rm, Write_File::write_file::write_file};


pub struct AgentContext {
    /// Immutable sandbox root
    pub workspace: PathBuf,
    /// Mutable current working directory
    pub cwd: PathBuf,
}

impl AgentContext{
    pub fn new(ws:PathBuf) ->Self{
        Self { workspace: ws.clone(), cwd: ws}
    }

}


pub trait Tool {
    fn name(&self) -> &'static str;
    
    fn description(&self) -> &'static str;

    fn execute(
        &self,
        ctx:&mut AgentContext,
        args: serde_json::Value,
    ) -> anyhow::Result<String>;
}


pub struct ToolRegistry {
    ctx: AgentContext,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new(ws:&PathBuf) -> anyhow::Result<Self> {
        let ws = ws.canonicalize()?;

        Ok(Self {
            ctx: AgentContext {
                workspace: ws.clone(),
                cwd: ws,
            },
            tools: HashMap::new(),
        })
    }

    pub fn get_all(&self) -> String{
        let mut buff = String::new();
        self.tools.iter().for_each(|x| {
            buff += x.0.as_str();
            buff += " ";
        });

        buff.trim().to_string()
    }


    pub fn register<T: Tool + 'static>(
        &mut self,
        tool: T,
    ) {
        self.tools
            .insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn execute(
        &mut self,
        name: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<String> {

        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown tool"))?;

        tool.execute(&mut self.ctx,args)
    }
}


    pub fn mode_to_string(mode: u32) -> String {
        let mut s = String::with_capacity(10);
        s.push(match mode & 0o170000 {
            0o100000 => '-', // Regular file
            0o040000 => 'd', // Directory
            0o120000 => 'l', // Symlink
            0o020000 => 'c', // Character device
            0o060000 => 'b', // Block device
            0o010000 => 'p', // FIFO/Pipe
            0o140000 => 's', // Socket
            _ => '?',
        });
        const BITS: &[(u32, char)] = &[
            (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
            (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
            (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
        ];
        for &(bit, ch) in BITS {
            s.push(if mode & bit != 0 { ch } else { '-' });
        }
        if mode & 0o4000 != 0 {
            s.replace_range(3..4, if mode & 0o100 != 0 { "s" } else { "S" });
        }
        if mode & 0o2000 != 0 {
            s.replace_range(6..7, if mode & 0o010 != 0 { "s" } else { "S" });
        }
        if mode & 0o1000 != 0 {
            s.replace_range(9..10, if mode & 0o001 != 0 { "t" } else { "T" });
        }
        s
    }

    pub fn estimate_tokens(s: &str) -> usize {
        (s.len() + 3) / 4
    }

    pub fn truncate_head_tail(s: &str, head: usize, tail: usize) -> String {
        let lines: Vec<_> = s.lines().collect();

        if lines.len() <= head + tail {
            return lines.join("\n");
        }

        let mut out = String::new();

        out.push_str(&lines[..head].join("\n"));
        out.push_str("\n\n... OUTPUT TRUNCATED ...\n\n");
        out.push_str(&lines[lines.len()-tail..].join("\n"));

        out
    }


    pub fn insert_tools(reg: &mut ToolRegistry){
        reg.register(cargo); 
        reg.register(cat); 
        reg.register(cd); 
        reg.register(mkdir); 
        reg.register(touch); 
        reg.register(find); 
        reg.register(Grep); 
        reg.register(ls); 
        reg.register(edit); 
        reg.register(pwd); 
        reg.register(rm); 
        reg.register(write_file); 

    }


}