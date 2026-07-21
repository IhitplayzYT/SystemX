use sled::{Db, Tree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub path: String,
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
    config: VectorStoreConfig,
}

impl VectorStore {
    pub fn new(config: VectorStoreConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = format!("{}/{}", config.path, config.collection_name);
        let db = sled::open(&db_path)?;
        
        let vectors_tree = db.open_tree("vectors")?;
        let metadata_tree = db.open_tree("metadata")?;
        
        Ok(VectorStore {
            db,
            vectors_tree,
            metadata_tree,
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

    pub fn insert(
        &self,
        id: Option<String>,
        vector: Vec<f32>,
        payload: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let point_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Store vector
        let vector_bytes = bincode::serialize(&vector)?;
        self.vectors_tree.insert(point_id.as_bytes(), vector_bytes)?;
        
        // Store metadata
        if let Some(metadata) = payload {
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            self.metadata_tree.insert(point_id.as_bytes(), metadata_bytes)?;
        }
        
        self.db.flush()?;
        Ok(point_id)
    }

    pub fn insert_batch(
        &self,
        points: Vec<(Option<String>, Vec<f32>, Option<HashMap<String, serde_json::Value>>)>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut ids = Vec::new();
        
        for (id, vector, payload) in points {
            let point_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
            
            let vector_bytes = bincode::serialize(&vector)?;
            self.vectors_tree.insert(point_id.as_bytes(), vector_bytes)?;
            
            if let Some(metadata) = payload {
                let metadata_bytes = serde_json::to_vec(&metadata)?;
                self.metadata_tree.insert(point_id.as_bytes(), metadata_bytes)?;
            }
            
            ids.push(point_id);
        }
        
        self.db.flush()?;
        Ok(ids)
    }

    pub fn get(&self, id: &str) -> Result<Option<Vec<f32>>, Box<dyn std::error::Error>> {
        if let Some(vector_bytes) = self.vectors_tree.get(id.as_bytes())? {
            let vector = bincode::deserialize(&vector_bytes)?;
            Ok(Some(vector))
        } else {
            Ok(None)
        }
    }

    pub fn get_metadata(&self, id: &str) -> Result<Option<HashMap<String, serde_json::Value>>, Box<dyn std::error::Error>> {
        if let Some(metadata_bytes) = self.metadata_tree.get(id.as_bytes())? {
            let metadata = serde_json::from_slice(&metadata_bytes)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    pub fn search(
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
            
            // Check metadata filter
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
        self.db.flush()?;
        Ok(())
    }

    pub fn delete_batch(&self, ids: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for id in ids {
            self.vectors_tree.remove(id.as_bytes())?;
            self.metadata_tree.remove(id.as_bytes())?;
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
        self.db.flush()?;
        Ok(())
    }

    pub fn recreate_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.clear()?;
        Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}

