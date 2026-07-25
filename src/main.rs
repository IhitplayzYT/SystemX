use std::{collections::{HashMap, HashSet}, fs};
use regex::Regex;

use crate::{chunker::{char::chunk::CharChunker, chunker::{Chunk, Chunker, t_Chunk}, colon::chunk::ColonChunker, line::chunk::LineChunker, nchar::chunk::NCharChunker, nline::chunk::NLineChunker, npara::chunk::NParaChunker, nword::chunk::NWordChunker, para::chunk::ParaChunker, semantic::chunk::SemanticChunker, sentence::chunk::SentenceChunker, word::chunk::WordChunker}, helper::Helper::{CLI, COLLECTION_N, VSTORE_N, unwrap_dirs}, vstore::{embed::embed::{generate_embedding, EmbedMethod}, vstore::Vstore::{VectorStore, VectorStoreConfig}}};

mod chunker;
mod ingestors;
mod model;
mod tools;
mod vstore;
mod helper;

fn main() {
    let mut clargs = CLI::new();
    CLI::Parse_Args(&mut clargs);
    if clargs.query.is_empty(){
        panic!("No query provided")
    }
    if clargs.query.is_empty(){
        panic!("No query provided");
    }
    if clargs.debug{
        println!("{clargs:?}");
    }

    unwrap_dirs(clargs.srcdir,&mut clargs.srcfile);
    let src_files: HashSet<_> = clargs.srcfile.iter().collect();
    let history = fs::read_to_string(clargs.context_file.unwrap()).unwrap();
    
    let chunker:Box<dyn t_Chunk> = match clargs.chunker{
    Chunker::WORD => Box::new(WordChunker),
    Chunker::LINE => Box::new(LineChunker),
    Chunker::PARAGRAPH => Box::new(ParaChunker),
    Chunker::SENTENCE => Box::new(SentenceChunker),
    Chunker::COLON => Box::new(ColonChunker),
    Chunker::SEMANTIC => Box::new(SemanticChunker),
    Chunker::CHAR(x) => Box::new(CharChunker{ ch:x as u8}),
    Chunker::NWORD(x) => Box::new(NWordChunker{n:x}),
    Chunker::NLINE(x) => Box::new(NLineChunker{n:x}),
    Chunker::NPARA(x) => Box::new(NParaChunker{n:x}),
    Chunker::NCHAR(ch,x) => Box::new(NCharChunker{n:x,ch})
    };

    let mut chunk_map: HashMap<&String,Vec<Chunk>> = HashMap::new();
    for i in &src_files{
        chunk_map.insert(*i,chunker.chunk(&fs::read_to_string(i).unwrap()));
    }
    let (vstore_path,collection_name) = ( if let Some(x) = clargs.vstore {x} else{VSTORE_N.to_string()}, if let Some(x) = clargs.collection {x} else{COLLECTION_N.to_string()});

    let vstore_config = VectorStoreConfig {
        path:vstore_path.clone(),
        collection_name:collection_name.clone()
    };
    let vstore = VectorStore::new(vstore_config).expect("Failed to create VStore");

    for (file_path, chunks) in &chunk_map {
        for (chunk_index, chunk_text) in chunks.iter().enumerate() {
            let embedding = generate_embedding(chunk_text, clargs.embed_model.unwrap_or(EmbedMethod::Semantic), clargs.vdim, clargs.window_len.unwrap_or(3));
            let mut metadata = HashMap::new();
            metadata.insert("file_path".to_string(), serde_json::json!(file_path));
            metadata.insert("chunk_index".to_string(), serde_json::json!(chunk_index));
            metadata.insert("chunk_count".to_string(), serde_json::json!(chunks.len()));
            vstore.insert(
                None,
                embedding,
                Some(metadata),
                Some(chunk_text.clone())
            ).expect("Failed to insert chunk into VectorStore");
        }
    }
    
    println!("Inserted {} chunks successfully \nVstore: {}\nCollection: {}", chunk_map.values().map(|v| v.len()).sum::<usize>(),&vstore_path[..],&collection_name[..]);

        




    


    

    
    

    


}
