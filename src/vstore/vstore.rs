pub mod Vstore{
use sled::{Db, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub path: String, // Where we store the embedded store for resilience
    pub collection_name: String, 
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        VectorStoreConfig {
            path: "./data/vstore".to_string(),
            collection_name: "default_collection".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorStore {
    db: Db,
    vectors_tree: Tree,
    metadata_tree: Tree,
    text_tree: Tree,
    config: VectorStoreConfig,
}

impl VectorStore {
    pub fn new(config: VectorStoreConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = format!("{}/{}", config.path, config.collection_name);
        let db = sled::open(&db_path)?;
        
        let vectors_tree = db.open_tree("vectors")?;
        let metadata_tree = db.open_tree("metadata")?;
        let text_tree = db.open_tree("text")?;
        
        Ok(VectorStore {
            db,
            vectors_tree,
            metadata_tree,
            text_tree,
            config,
        })
    }

    pub fn from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = VectorStoreConfig {
            path: path.to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    // Insert single data into vstore
    pub fn insert(
        &self,
        id: Option<String>,
        vector: Vec<f32>,
        meta: Option<HashMap<String, serde_json::Value>>,
        text: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let point_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Store vector
        let vector_bytes = bincode::serialize(&vector)?;
        self.vectors_tree.insert(point_id.as_bytes(), vector_bytes)?;
        
        // Store metadata
        if let Some(metadata) = meta {
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            self.metadata_tree.insert(point_id.as_bytes(), metadata_bytes)?;
        }
        
        // Store text for BM25 retrieval
        if let Some(text_content) = text {
            self.text_tree.insert(point_id.as_bytes(), text_content.as_bytes())?;
        }
        
        self.db.flush()?;
        Ok(point_id)
    }

    // Insert list of data into vstore
    pub fn insert_batch(
        &self,
        points: Vec<(Option<String>, Vec<f32>, Option<HashMap<String, serde_json::Value>>, Option<String>)>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut ids = Vec::new();
        
        for (id, vector, meta, text) in points {
            let point_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
            
            let vector_bytes = bincode::serialize(&vector)?;
            self.vectors_tree.insert(point_id.as_bytes(), vector_bytes)?;
            
            if let Some(metadata) = meta {
                let metadata_bytes = serde_json::to_vec(&metadata)?;
                self.metadata_tree.insert(point_id.as_bytes(), metadata_bytes)?;
            }
            
            if let Some(text_content) = text {
                self.text_tree.insert(point_id.as_bytes(), text_content.as_bytes())?;
            }
            
            ids.push(point_id);
        }
        
        self.db.flush()?;
        Ok(ids)
    }

    // Search a single id for embedding
    pub fn get(&self, id: &str) -> Result<Option<Vec<f32>>, Box<dyn std::error::Error>> {
        if let Some(vector_bytes) = self.vectors_tree.get(id.as_bytes())? {
            let vector = bincode::deserialize(&vector_bytes)?;
            Ok(Some(vector))
        } else {
            Ok(None)
        }
    }

    // Search a single id for metadata 
    pub fn get_metadata(&self, id: &str) -> Result<Option<HashMap<String, serde_json::Value>>, Box<dyn std::error::Error>> {
        if let Some(metadata_bytes) = self.metadata_tree.get(id.as_bytes())? {
            let metadata = serde_json::from_slice(&metadata_bytes)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    // Search a single id for text content
    pub fn get_text(&self, id: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(text_bytes) = self.text_tree.get(id.as_bytes())? {
            let text = String::from_utf8(text_bytes.to_vec())?;
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    pub fn get_top_k(
        &self,
        query_vector: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let mut results: Vec<SearchResult> = Vec::new();
        
        for item in self.vectors_tree.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            let vector: Vec<f32> = bincode::deserialize(&value)?;
            
            let similarity = cosine_similarity(query_vector, &vector);
            results.push(SearchResult {
                id,
                score: similarity,
            });
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        Ok(results)
    }

    pub fn search_with_metadata_filter(
        &self,
        query_vector: &[f32],
        top_k: usize,
        metadata_filter: HashMap<String, String>,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let mut results: Vec<SearchResult> = Vec::new();
        
        for item in self.vectors_tree.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            let vector: Vec<f32> = bincode::deserialize(&value)?;
            
            if let Some(metadata_bytes) = self.metadata_tree.get(&key)? {
                let metadata: HashMap<String, serde_json::Value> = serde_json::from_slice(&metadata_bytes)?;
                
                let matches = metadata_filter.iter().all(|(k, v)| {
                    metadata.get(k)
                        .and_then(|val| val.as_str())
                        .map(|s| s == v)
                        .unwrap_or(false)
                });
                
                if matches {
                    let similarity = cosine_similarity(query_vector, &vector);
                    results.push(SearchResult {
                        id,
                        score: similarity,
                    });
                }
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        Ok(results)
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.vectors_tree.remove(id.as_bytes())?;
        self.metadata_tree.remove(id.as_bytes())?;
        self.text_tree.remove(id.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn delete_batch(&self, ids: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for id in ids {
            self.vectors_tree.remove(id.as_bytes())?;
            self.metadata_tree.remove(id.as_bytes())?;
            self.text_tree.remove(id.as_bytes())?;
        }
        self.db.flush()?;
        Ok(())
    }

    pub fn update(
        &self,
        id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(vec) = vector {
            let vector_bytes = bincode::serialize(&vec)?;
            self.vectors_tree.insert(id.as_bytes(), vector_bytes)?;
        }
        
        if let Some(metadata) = payload {
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            self.metadata_tree.insert(id.as_bytes(), metadata_bytes)?;
        }
        
        self.db.flush()?;
        Ok(())
    }

    pub fn count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.vectors_tree.len())
    }

    pub fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.vectors_tree.clear()?;
        self.metadata_tree.clear()?;
        self.text_tree.clear()?;
        self.db.flush()?;
        Ok(())
    }

    pub fn recreate_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.clear()?;
        Ok(())
    }

    pub fn semantic_retrieval(
        &self,
        query_vector: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        self.get_top_k(query_vector, top_k)
    }

    // Keyword retrieval using BM25 scoring
    pub fn keyword_retrieval(
        &self,
        query: &str,
        top_k: usize,
        k1: f32,
        b: f32,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let query_tokens = tokenize(query);
        let mut corpus_stats = self.calculate_corpus_stats()?;
        let mut results: Vec<SearchResult> = Vec::new();
        for item in self.text_tree.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            let text = String::from_utf8(value.to_vec())?;
            let doc_tokens = tokenize(&text);
            let doc_length = doc_tokens.len() as f32;
            let avg_doc_length = corpus_stats.avg_doc_length;
            let mut bm25_score = 0.0;
            for token in &query_tokens {
                let tf = doc_tokens.iter().filter(|t| *t == token).count() as f32;
                if tf > 0.0 {
                    let df = corpus_stats.doc_freqs.get(token).copied().unwrap_or(1.0);
                    let idf = ((corpus_stats.total_docs as f32 - df + 0.5) / (df + 0.5) + 1.0).ln_1p();
                    let numerator = tf * (k1 + 1.0);
                    let denominator = tf + k1 * (1.0 - b + b * (doc_length / avg_doc_length));
                    bm25_score += idf * (numerator / denominator);
                }
            }
            
            results.push(SearchResult {
                id,
                score: bm25_score,
            });
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        Ok(results)
    }

    // Hybrid retrieval combining semantic and BM25 scores
    pub fn hybrid_retrieval(
        &self,
        query_vector: &[f32],
        query_text: &str,
        top_k: usize,
        semantic_weight: f32,
        keyword_weight: f32,
        k1: f32,
        b: f32,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let semantic_results = self.semantic_retrieval(query_vector, usize::MAX)?;
        let keyword_results = self.keyword_retrieval(query_text, usize::MAX, k1, b)?;
        
        let mut semantic_scores: HashMap<String, f32> = HashMap::new();
        for result in semantic_results {
            semantic_scores.insert(result.id, result.score);
        }
        
        let mut keyword_scores: HashMap<String, f32> = HashMap::new();
        for result in keyword_results {
            keyword_scores.insert(result.id, result.score);
        }
        
        // Normalize scores
        let max_semantic = semantic_scores.values().cloned().fold(0.0_f32, f32::max);
        let max_keyword = keyword_scores.values().cloned().fold(0.0_f32, f32::max);
        
        let mut all_ids: HashSet<String> = HashSet::new();
        all_ids.extend(semantic_scores.keys().cloned());
        all_ids.extend(keyword_scores.keys().cloned());
        
        let mut hybrid_results: Vec<SearchResult> = Vec::new();
        
        for id in all_ids {
            let semantic_score = semantic_scores.get(&id).copied().unwrap_or(0.0);
            let keyword_score = keyword_scores.get(&id).copied().unwrap_or(0.0);
            
            let normalized_semantic = if max_semantic > 0.0 { semantic_score / max_semantic } else { 0.0 };
            let normalized_keyword = if max_keyword > 0.0 { keyword_score / max_keyword } else { 0.0 };
            
            let hybrid_score = (normalized_semantic * semantic_weight) + (normalized_keyword * keyword_weight);
            
            hybrid_results.push(SearchResult {
                id,
                score: hybrid_score,
            });
        }
        
        hybrid_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hybrid_results.truncate(top_k);
        
        Ok(hybrid_results)
    }

    fn calculate_corpus_stats(&self) -> Result<CorpusStats, Box<dyn std::error::Error>> {
        let mut doc_freqs: HashMap<String, f32> = HashMap::new();
        let mut total_doc_length: f32 = 0.0;
        let mut total_docs = 0;
        
        for item in self.text_tree.iter() {
            let (_key, value) = item?;
            let text = String::from_utf8(value.to_vec())?;
            let tokens = tokenize(&text);
            
            total_doc_length += tokens.len() as f32;
            total_docs += 1;
            
            let unique_tokens: HashSet<&String> = tokens.iter().collect();
            for token in unique_tokens {
                *doc_freqs.entry(token.clone()).or_insert(0.0) += 1.0;
            }
        }
        
        let avg_doc_length = if total_docs > 0 { total_doc_length / total_docs as f32 } else { 0.0 };
        
        Ok(CorpusStats {
            total_docs,
            avg_doc_length,
            doc_freqs,
        })
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

// Tokenize text into lowercase words, removing punctuation
fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}

#[derive(Debug, Clone)]
struct CorpusStats {
    total_docs: usize,
    avg_doc_length: f32,
    doc_freqs: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}



}