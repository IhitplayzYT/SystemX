
#[derive(Debug,Clone)]
pub enum Chunker{
    WORD,
    LINE,
    PARAGRAPH,
    SENTENCE,
    COLON,
    SEMANTIC,
    FXNBLOCK,
    NWORD(usize),
    NLINE(usize),
    NPARA(usize),
}

impl From<String> for Chunker{
fn from(value: String) -> Self {
    let mut ret = match &value[..]{
    "word" => Chunker::WORD,
    "line" => Chunker::LINE,
    "para" => Chunker::PARAGRAPH,
    "sentence" => Chunker::SENTENCE,
    "colon" => Chunker::COLON,
    "semantic" => Chunker::SEMANTIC,
    "fxn" => Chunker::FXNBLOCK,
    _ => {Chunker::NLINE(usize::MAX)}
    };
    if let Chunker::NLINE(x) = ret && x == usize::MAX{
        let (st,ed) = (value.find("(").unwrap(),value.find(")").unwrap());
        let val = value[st+1..ed].parse::<usize>().expect("Usize expected");
        ret = match &value[..st]{
            "nline" => Chunker::NLINE(val),
            "npara" => Chunker::NPARA(val),
            "nword" => Chunker::NWORD(val),
            _ => Chunker::LINE
        };   
    }
    ret
}
}

pub type Chunk = String;


pub trait t_Chunk {
    fn chunk(&self,txt:&str) -> Vec<Chunk>;   
}