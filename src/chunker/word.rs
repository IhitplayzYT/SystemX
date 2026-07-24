pub mod chunk{
    use crate::chunker::chunker::{t_Chunk,Chunk};
pub struct WordChunker;

impl t_Chunk for WordChunker{

    fn chunk(&self,txt:&str) -> Vec<Chunk> {
        let txt = String::from_utf8(txt.bytes().filter(|z| {(z >= &b'a' && z <= &b'z') || (z >= &b'A' && z <= &b'Z') || (z >= &b'0' && z <= &b'9') || z == &b' ' || z == &b'\n' || z == &b'\t'}).collect::<Vec<u8>>()).unwrap();
        txt.split(" ").map(|x| x.to_string()).collect()
    }    

}
}