pub mod chunk{
    use crate::chunker::chunker::{t_Chunk,Chunk};
    use std::collections::HashMap;
    use regex::Regex;

pub struct SemanticChunker;

impl t_Chunk for SemanticChunker{

    fn chunk(&self,txt:&str) -> Vec<Chunk> {
        let txt = String::from_utf8(txt.bytes().filter(|z| {(z >= &b'a' && z <= &b'z') || (z >= &b'A' && z <= &b'Z') || (z >= &b'0' && z <= &b'9') || z == &b' ' || z == &b'\n' || z == &b'\t' || z == &b'.' || z == &b',' || z == &b'?' || z == &b'!'}).collect::<Vec<u8>>()).unwrap();
        
        // Split into sentences
        let sentences: Vec<&str> = txt.split(&['.', '!', '?'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        
        if sentences.len() <= 1 {
            return vec![txt];
        }

        // Generate embeddings for each sentence using TF-IDF-like approach
        let embeddings = generate_embeddings(&sentences);
        
        // Group sentences based on semantic similarity
        let chunks = group_by_similarity(&sentences, &embeddings, 0.3); // 0.3 similarity threshold
        
        chunks.into_iter().map(|c| c.join(". ").trim().to_string()).filter(|c| !c.is_empty()).collect()
    }    

}

fn generate_embeddings(sentences: &[&str]) -> Vec<HashMap<String, f32>> {
    // Build vocabulary and document frequency
    let mut vocab: HashMap<String, usize> = HashMap::new();
    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    
    for (i, sentence) in sentences.iter().enumerate() {
        let words = tokenize(sentence);
        let mut seen_words: HashMap<String, bool> = HashMap::new();
        
        for word in &words {
            if !vocab.contains_key(word) {
                vocab.insert(word.clone(), vocab.len());
            }
            if !seen_words.contains_key(word) {
                *doc_freq.entry(word.clone()).or_insert(0) += 1;
                seen_words.insert(word.clone(), true);
            }
        }
    }
    
    let total_docs = sentences.len() as f32;
    
    // Generate TF-IDF vectors for each sentence
    sentences.iter().map(|sentence| {
        let words = tokenize(sentence);
        let mut tf: HashMap<String, f32> = HashMap::new();
        
        // Calculate term frequency
        for word in &words {
            *tf.entry(word.clone()).or_insert(0.0) += 1.0;
        }
        
        // Normalize TF and multiply by IDF
        let total_terms = words.len() as f32;
        let mut embedding: HashMap<String, f32> = HashMap::new();
        
        for (word, count) in tf {
            let tf_normalized = count / total_terms;
            let idf = (total_docs / (*doc_freq.get(&word).unwrap_or(&1) as f32)).ln_1p();
            embedding.insert(word, tf_normalized * idf);
        }
        
        embedding
    }).collect()
}

fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

fn cosine_similarity(vec1: &HashMap<String, f32>, vec2: &HashMap<String, f32>) -> f32 {
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;
    
    // Calculate dot product and norms
    for (word, val1) in vec1 {
        let val2 = vec2.get(word).unwrap_or(&0.0);
        dot_product += val1 * val2;
        norm1 += val1 * val1;
    }
    
    for (_, val2) in vec2 {
        norm2 += val2 * val2;
    }
    
    let norm1 = norm1.sqrt();
    let norm2 = norm2.sqrt();
    
    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot_product / (norm1 * norm2)
    }
}

fn group_by_similarity<'a>(sentences: &'a [&str], embeddings: &[HashMap<String, f32>], threshold: f32) -> Vec<Vec<&'a str>> {
    let mut chunks: Vec<Vec<&'a str>> = Vec::new();
    let mut current_chunk: Vec<&'a str> = Vec::new();
    
    if sentences.is_empty() {
        return chunks;
    }
    
    current_chunk.push(sentences[0]);
    
    for i in 1..sentences.len() {
        let prev_embedding = &embeddings[i - 1];
        let curr_embedding = &embeddings[i];
        
        let similarity = cosine_similarity(prev_embedding, curr_embedding);
        
        if similarity >= threshold {
            current_chunk.push(sentences[i]);
        } else {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk = Vec::new();
            }
            current_chunk.push(sentences[i]);
        }
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    chunks
}
}
