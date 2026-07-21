pub mod chunk{
    use crate::chunker::chunker::{t_Chunk,Chunk};

pub struct NCharChunker{
pub n:usize,
pub ch:u8,
}

impl Default for NCharChunker{
    fn default() -> Self {
        Self { n: 2 , ch:b'.'}
    }
}


impl t_Chunk for NCharChunker{
    fn chunk(&self,txt:&str) -> Vec<Chunk> {
        let txt = String::from_utf8(txt.bytes().filter(|z| {(z >= &b'a' && z <= &b'z') || (z >= &b'A' && z <= &b'Z') || (z >= &b'0' && z <= &b'9') || z == &b' ' || z == &b'\n' || z == &b'\t'}).collect::<Vec<u8>>()).unwrap();
        let words: Vec<_> = txt.split(self.ch as char).collect();
        words.chunks(self.n).map(|chunk| chunk.join(" ")).collect()
    }   

}
}