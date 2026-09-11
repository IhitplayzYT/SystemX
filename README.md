# SystemX

A sophisticated AI agent framework for document processing, retrieval-augmented generation (RAG), and tool-based autonomous execution. SystemX enables intelligent document analysis and interaction through local LLM integration (Ollama).

## Features

- **Multi-Format Document Ingestion**: Support for PDF, DOCX, Excel, CSV, HTML, and plain text files
- **Advanced Chunking Strategies**: Word, line, paragraph, sentence, semantic, and custom N-based chunking
- **Vector Storage**: Local embedded database using sled for persistent vector storage
- **Multiple Embedding Methods**: Semantic, Cluster, TF-IDF, and Graph-based embeddings
- **AI Agent with Tool Execution**: Autonomous agent capable of executing file operations and commands
- **Hybrid Retrieval**: Combines semantic similarity with BM25 keyword search
- **RAG Tools**: Specialized tools for retrieval-augmented generation workflows
- **User Permission System**: Safe command execution with user confirmation

## Why SystemX?

SystemX addresses the need for a local, privacy-preserving AI agent that can:
- Process and understand large document collections
- Retrieve relevant information using multiple search strategies
- Execute file system operations autonomously
- Work entirely offline with local LLM models
- Provide transparent tool usage with user oversight

## Dependencies

SystemX requires the following Rust dependencies:

```toml
regex = "1.10"
sled = "0.34"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1"
path-clean = "1"
walkdir = "2"
grep-searcher = "0.1"
grep-regex = "0.1"
grep-printer = "0.2"
reqwest = { version = "0.12", features = ["blocking", "json"] }
scraper = "0.24"
pdf-extract = "0.8"
zip = "4"
quick-xml = "0.38"
calamine = "0.31"
csv = "1.3"
```

### External Requirements

- **Ollama**: Local LLM server (default: http://localhost:11434)
- **Rust**: Edition 2024 or later

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd SystemX
```

2. Install Ollama and pull a model (e.g., llama3.2):
```bash
ollama pull llama3.2
```

3. Build the project:
```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
cargo run -- --query="your question" --sysprompt="You are a helpful assistant" --srcdir="./documents"
```

### Command Line Arguments

| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--help` | `-h` | Show help message | - |
| `--debug` | `-d` | Enable debug mode | false |
| `--srcdir=PATH` | - | Source directory to process | - |
| `--srcfile=PATH` | - | Specific source file to process | - |
| `--chunker=TYPE` | `-s` | Chunking strategy | line |
| `--query=TEXT` | `-q` | Query/question for the agent | - |
| `--context=PATH` | `-c` | Context file path | - |
| `--model=NAME` | `-m` | Ollama model name | llama3.2 |
| `--temp=FLOAT` | `-t` | Temperature (0.0-2.0) | 0.4 |
| `--vdim=SIZE` | `-n` | Vector embedding dimensions | 384 |
| `--vstore=PATH` | `-v` | Vector store path | ./data/vstore |
| `--collection=NAME` | - | Collection name | document_chunks |
| `--embed-model=TYPE` | `-e` | Embedding method | semantic |
| `--winlen=SIZE` | `-w` | Window length for cluster embedding | 3 |
| `--url=URL` | `-u` | Ollama endpoint URL | http://localhost:11434 |
| `--min=SIZE` | - | Minimum context tokens | 0 |
| `--max=SIZE` | - | Maximum context tokens (0=unbounded) | 100000 |
| `--maxout=SIZE` | - | Maximum output tokens | 8192 |
| `--root=PATH` | - | Root directory for agent | current dir |
| `--steps=COUNT` | `-s` | Maximum agent steps | 20 |
| `--memory=PATH` | - | Memory file path | - |
| `--sysprompt=TEXT` | - | System prompt for agent | - |

### Chunking Strategies

- `word` - Chunk by words
- `line` - Chunk by lines (default)
- `para` - Chunk by paragraphs
- `sentence` - Chunk by sentences
- `colon` - Chunk by colons
- `semantic` - Semantic chunking
- `char(X)` - Chunk by character X
- `nchar(C,N)` - N characters with delimiter C
- `nline(N)` - N lines per chunk
- `npara(N)` - N paragraphs per chunk
- `nword(N)` - N words per chunk

### Embedding Methods

- `semantic` - Semantic embedding with positional bias
- `cluster` - Co-occurrence cluster embedding
- `tf-idf` / `tfidf` - TF-IDF based embedding
- `graph` - Word adjacency graph embedding

## Examples

### Example 1: Process a directory of documents

```bash
cargo run -- \
  --query="What are the main topics discussed?" \
  --sysprompt="You are a document analysis assistant" \
  --srcdir="./docs" \
  --chunker="paragraph" \
  --embed-model="semantic"
```

### Example 2: Process a specific file with custom chunking

```bash
cargo run -- \
  --query="Summarize the key points" \
  --sysprompt="You are a helpful summarizer" \
  --srcfile="./report.pdf" \
  --chunker="nline(10)" \
  --vdim=512
```

### Example 3: Use cluster embedding with custom window

```bash
cargo run -- \
  --query="Find related concepts" \
  --sysprompt="You are a research assistant" \
  --srcdir="./papers" \
  --chunker="sentence" \
  --embed-model="cluster" \
  --winlen=5
```

### Example 4: Debug mode with verbose output

```bash
cargo run -- \
  --debug \
  --query="Analyze the code structure" \
  --sysprompt="You are a code reviewer" \
  --srcdir="./src" \
  --chunker="line"
```

### Example 5: Custom Ollama configuration

```bash
cargo run -- \
  --query="Explain the architecture" \
  --sysprompt="You are a technical writer" \
  --srcdir="./docs" \
  --url="http://localhost:11434" \
  --model="llama3.2" \
  --temp=0.7
```

## Architecture

### Core Components

1. **Chunker Module** (`src/chunker/`): Various text chunking strategies
2. **Ingestors Module** (`src/ingestors/`): Document format parsers
3. **Model Module** (`src/model/`): AI agent and LLM integration
4. **Tools Module** (`src/tools/`): File system and utility tools
5. **VStore Module** (`src/vstore/`): Vector storage and retrieval
6. **Helper Module** (`src/helper.rs`): CLI argument parsing and utilities

### Available Tools

The agent has access to the following tools:
- `cargo` - Execute cargo commands
- `cat` - Read file contents
- `cd` - Change directory
- `mkdir` - Create directory
- `touch` - Create file
- `find` - Find files
- `grep` - Search in files
- `ls` - List directory contents
- `edit` - Modify file
- `pwd` - Print working directory
- `rm` - Remove directory
- `write_file` - Write to file
- `Bash` - Execute shell commands (with permission)
- `Get_html` - Fetch HTML content
- `Get_html_selectors` - Extract HTML elements
- RAG tools for vector search

## Vector Storage

SystemX uses sled as the embedded database for vector storage. The vector store supports:
- Insertion of single or batch vectors with metadata
- Cosine similarity search
- Metadata filtering
- BM25 keyword retrieval
- Hybrid retrieval (semantic + keyword)
- CRUD operations

## License

GPL-3.0-only - See LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust conventions (warnings are acceptable during development)
- New features include tests
- Documentation is updated accordingly

## Notes

- The project uses Rust 2024 edition
- Compiler warnings about naming conventions (non_snake_case) are acceptable and do not affect functionality
- The agent requires user confirmation for command execution
- Vector embeddings are generated locally without external API calls
