pub mod Agent{
use std::{
     fmt::Display, path::PathBuf, sync::{Arc, RwLock, atomic::{AtomicBool, Ordering},},
};
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};

use crate::{helper::Helper::{END_POINT, MODEL}, tools::tools::Tools::{Tool, ToolRegistry}};

static mut Chat_History: LazyLock<RwLock<Vec<Message>>> = LazyLock::new(|| RwLock::new(vec![]));

pub struct Agent {
    pub cwd:PathBuf,
    pub model: Box<dyn LLM>,
    pub tools: ToolRegistry,
    pub memory: Memory,
    pub steps: usize,    
    pub config: AgentConfig,
    pub state: AgentState,
    pub cancelled: Arc<AtomicBool>,

}

impl Display for Agent{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"===== Agent Details =====
        Workspace: {:?}
        LLM: {}
        Available Tools: {:?}
        Memory: {}
        Steps: {}
        AgentConfig: {:?}
        SystemPrompt: {}
        =================",&self.cwd,self.model.name(),self.tools.get_all(),if self.memory.messages.is_empty(){"Empty"} else{"Populated"},self.steps,self.config,if let Message::System(x) = &self.memory.messages[0]{&x}else{"Fatal"})
    }

}

impl Agent{
    pub fn new(cwd:PathBuf,model: Option<Box<dyn LLM>>,tools: Option<ToolRegistry>,memory:Option<Memory>,steps:Option<usize>,config:Option<AgentConfig>) -> Self{
        Self { cwd:cwd.clone(), model: model.unwrap_or(Box::new(Ollama::new(None, None))), tools:tools.unwrap_or(ToolRegistry::new(&cwd).unwrap()), memory: memory.unwrap_or(Memory::new()), steps: steps.unwrap_or(10), config: config.unwrap_or(AgentConfig::default()), state: AgentState::Idle, cancelled: Arc::new(AtomicBool::new(false))}
    }

}


#[derive(Clone,Debug)]
pub struct AgentConfig {
    pub max_steps: usize,
    pub min_context_tokens:usize,
    pub max_context_tokens: usize,
    pub max_output_tokens: usize,
    pub temperature: f32,
}



impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            min_context_tokens: 0,
            max_context_tokens: 200000, // Ensures massive context
            max_output_tokens: 8192,
            temperature: 0.4, // For coding we keep temp low to avoid creativity and p;refer working over creative
        }
    }
}

impl AgentConfig{
    pub fn new(ms:Option<usize>,mct:Option<usize>,mact:Option<usize>,maot:Option<usize>,t:Option<f32>) -> Self{
        Self { max_steps: ms.unwrap_or(0), min_context_tokens: mct.unwrap_or(0), max_context_tokens: mact.unwrap_or(100000), max_output_tokens: maot.unwrap_or(4096), temperature: t.unwrap_or(0.0) }
    }
}


impl Agent {

    pub fn min_new(
        cwd: PathBuf,
        model: Box<dyn LLM>,
    ) -> anyhow::Result<Self> {

        let cwd = cwd.canonicalize()?;

        Ok(Self {
            cwd: cwd.clone(),
            model,
            tools: ToolRegistry::new(&cwd).unwrap(),
            memory: Memory::new(),
            config: AgentConfig::default(),
            state: AgentState::Idle,
            steps: 0,
            cancelled: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn register_tool<T: Tool + 'static>(
        &mut self,
        tool: T,
    ) {
        self.tools.register(tool);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn run(&mut self, user_input: String) -> anyhow::Result<String> {
        self.memory.push_user(user_input);
        self.state = AgentState::Thinking;
        self.steps = 0;

        loop {
            if self.is_cancelled() {
                self.state = AgentState::Cancelled;
                return Ok("Agent execution cancelled".to_string());
            }
            if self.steps >= self.config.max_steps {
                self.state = AgentState::Finished;
                return Ok(format!("Max steps ({}) reached", self.config.max_steps));
            }
            let response = self.model.complete(&self.memory, &self.config)?;
            match response {
                ModelResponse::ToolCall { name, arguments } => {
                    self.state = AgentState::ExecutingTool;
                    self.steps += 1;

                    if name == "Command" {
                        println!("Agent wants to execute command: {:?}", arguments);
                        print!("Do you permit this? (Yes/y to confirm): ");
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();

                        let mut permission = String::new();
                        std::io::stdin().read_line(&mut permission).expect("Failed to read permission");
                        let permission = permission.trim().to_lowercase();

                        if permission != "yes" && permission != "y" {
                            let error_msg = "Command execution denied by user".to_string();
                            self.memory.push_tool(name, error_msg);
                            continue;
                        }
                    }

                    let tool_output = self.tools.execute(&name, arguments)?;
                    self.memory.push_tool(name, tool_output);

                }
                ModelResponse::Final(content) => {
                    self.memory.push_assistant(content.clone());
                    self.state = AgentState::Finished;
                    return Ok(content);
                }
            }
        }
    }

}

#[derive(Debug,Serialize,Deserialize)]
pub struct Memory {
    messages: Vec<Message>,
}

impl Memory {

    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push_user(
        &mut self,
        text: String,
    ) {
        self.messages.push(Message::User(text));
    }

    pub fn push_system(
        &mut self,
        text: String,
    ) {
        self.messages.push(Message::System(text));
    }

    pub fn push_tool(
        &mut self,
        tool: String,
        output: String,
    ) {
        self.messages.push(
            Message::Tool(tool, output)
        );
    }

    pub fn push_assistant(
        &mut self,
        text: String,
    ) {
        self.messages.push(
            Message::Assistant(text)
        );
    }

    fn unwrap_memory(&self) -> Vec<(&'static str,String)> {
        self.messages.iter().map(|m| {
            match m {
                Message::System(x) => ("system",x.clone()),
                Message::User(x) => ("user",x.clone()),
                Message::Assistant(x) => ("assistant",x.clone()),
                Message::Tool(name, output) => ("tool",format!("{name}\n{output}")),
            }

        }).collect()
    }

    fn to_ollama(&self) -> Vec<OllamaMessage> {
        self.messages.iter().map(|m| {
            match m {
                Message::System(x) => OllamaMessage {
                    role: "system".into(),content: x.clone(),
                },
                Message::User(x) => OllamaMessage {
                    role: "user".into(),content: x.clone(),
                },
                Message::Assistant(x) => OllamaMessage {
                    role: "assistant".into(),content: x.clone(),
                },
                Message::Tool(name, output) => OllamaMessage {
                    role: "tool".into(),content: format!("{name}\n{output}"),
                },
            }

        }).collect()
    }



}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Thinking,
    ExecutingTool,
    Finished,
    Cancelled,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    Tool(String,String),
}

impl Message{
    pub fn role(&self) -> &'static str{
        match self{
            Message::Assistant(_) => "assistant",
            Message::System(_) => "system",
            Message::User(_) => "user",
            Message::Tool(_,_) => "tool",
        }
    }


}


pub trait LLM {
    fn complete(
        &mut self,
        memory: &Memory,
        config: &AgentConfig,
    ) -> anyhow::Result<ModelResponse>;

    fn name(&self) -> &'static str;
}
pub struct Ollama {
    pub endpoint: String,
    pub model: String,
    pub client: reqwest::blocking::Client,
}

impl Ollama{
    pub fn new(ed:Option<String>,model:Option<String>) -> Self{
        Self { endpoint: ed.unwrap_or(END_POINT.to_string()), model: model.unwrap_or(MODEL.to_string()),client: reqwest::blocking::Client::new(),}
    }

}

impl LLM for Ollama {
    fn name(&self) -> &'static str {
        "Ollama"
    }

    fn complete(
        &mut self,
        memory: &Memory,
        config: &AgentConfig,
    ) -> anyhow::Result<ModelResponse> {
        let req = OllamaRequest{
            model: &self.model,
            messages: memory.to_ollama(),
            stream:false,
            temp: config.temperature,
        };
        let resp :OllamaResponse= self.client.post(format!("{}/api/chat",self.endpoint)).json(&req).send()?.error_for_status()?.json()?;

        parse_ollama_response(resp.message.content)
    }

}


// Weird Magic Json Syntax for serde conversions of json
#[derive(Deserialize)]
#[serde(tag="type")]
enum RawResponse {

    #[serde(rename="tool")]
    Tool {
        name: String,
        arguments: serde_json::Value,
    },

    #[serde(rename="final")]
    Final {
        content: String,
    },
}

pub fn parse_ollama_response(content:String) -> anyhow::Result<ModelResponse>{
    let r: RawResponse = serde_json::from_str(&content)?;

    Ok(match r {

        RawResponse::Tool {
            name,
            arguments,
        } => ModelResponse::ToolCall {
            name,
            arguments,
        },

        RawResponse::Final {
            content,
        } => ModelResponse::Final(content),
    })
}


#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    stream: bool,
    temp: f32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

pub enum ModelResponse {
    ToolCall {
        name: String,
        arguments: serde_json::Value,
    },
    Final(String),
}

}