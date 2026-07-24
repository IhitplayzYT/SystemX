
#[derive(Debug,Clone)]
pub enum Chunker{
    WORD,
    LINE,
    PARAGRAPH,
    SENTENCE,
    COLON,
    SEMANTIC,
    CHAR(char),
    NWORD(usize),
    NLINE(usize),
    NPARA(usize),
    NCHAR(u8,usize),
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
    _ => {Chunker::NLINE(usize::MAX)}
    };
    if value.starts_with("char"){

       let (st,ed) = (value.find("(").unwrap(),value.find(")").unwrap());
       return Chunker::CHAR(value[st+1..ed].parse::<char>().expect("Charecter expected"));
    }
    if value.starts_with("nchar"){
       let (st,ed,md) = (value.find("(").unwrap(),value.find(")").unwrap(),value.find(",").unwrap());
       return Chunker::NCHAR(value[st+1..md].parse::<u8>().expect("Expected a single charecter"), value[md+1..ed].parse::<usize>().expect("Usize expected"))
    }
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