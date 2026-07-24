use std::{collections::{HashMap, HashSet}, fs};

use crate::{chunker::{char::chunk::CharChunker, chunker::{Chunk, Chunker, t_Chunk}, colon::chunk::ColonChunker, line::chunk::LineChunker, nchar::chunk::NCharChunker, nline::chunk::NLineChunker, npara::chunk::NParaChunker, nword::chunk::NWordChunker, para::chunk::ParaChunker, semantic::chunk::SemanticChunker, sentence::chunk::SentenceChunker, word::chunk::WordChunker}, helper::Helper::{CLI, unwrap_dirs}};

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

    


    

    
    

    


}
