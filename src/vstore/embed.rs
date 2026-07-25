pub mod embed {
    use std::{collections::{HashMap, HashSet}};
    use regex::Regex;
    #[derive(Debug, Clone, Copy)]
    pub enum EmbedMethod {
        Semantic,
        Cluster,
        TfIdf,
        Graph,
    }

    impl From<String> for EmbedMethod{
        fn from(value: String) -> Self {
            match &value.to_lowercase()[..]{
                "semantic" => EmbedMethod::Semantic,
                "cluster" => EmbedMethod::Cluster,
                "tf-idf" | "tfidf" => EmbedMethod::TfIdf,
                "graph" => EmbedMethod::Graph,
                _ => EmbedMethod::Semantic
            }
        }

    }


    pub fn generate_embedding(text: &str, method: EmbedMethod,vdim:usize,win_len:usize) -> Vec<f32> {
        match method {
            EmbedMethod::Semantic => semantic_embedding(text,vdim),
            EmbedMethod::Cluster => cluster_embedding(text,vdim,win_len),
            EmbedMethod::TfIdf => tfidf_embedding(text,vdim),
            EmbedMethod::Graph => graph_embedding(text,vdim),
        }
    }

    pub fn semantic_embedding(text: &str,vdim:usize) -> Vec<f32> {
        let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
        let words: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_lowercase()).collect();
        let l = words.len();
        if words.is_empty() {
            return vec![0.0_f32; vdim];
        }
        
        let mut vocab: HashMap<String, usize> = HashMap::new();
        let mut tf: HashMap<String, f32> = HashMap::new();
        for i in &words {
            if !vocab.contains_key(i) {
                vocab.insert(i.clone(), vocab.len());
            }
            *tf.entry(i.clone()).or_insert(0.0) += 1.0;
        }
        let mut embedding = vec![0.0_f32; vocab.len().max(vdim)];        

       // Improves embeddings with positional bias weights
        for (w, c) in tf {
            if let Some(&idx) = vocab.get(&w) {
                if idx < embedding.len() {
                    let pos_ratio = 1.0 + (words.iter().position(|x| *x == w).unwrap_or(0) as f32 / words.len() as f32);
                    embedding[idx] = (c / l as f32) * pos_ratio;
                }
            }
        }
        normalize(&mut embedding);
        pad(&mut embedding, vdim);
        embedding
    }

    pub fn cluster_embedding(text: &str,vdim:usize,window_size:usize) -> Vec<f32> {
        let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
        let words: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_lowercase()).collect();
        // Since for a comparing we need atleast 2 words
        if words.len() < 2 {
            return vec![0.0; vdim];
        }
        let mut co_occurrence: HashMap<(String, String), f32> = HashMap::new();
        for i in 0..words.len() {
            let start = (i - window_size).max(0);
            let end = (i + window_size + 1).min(words.len());
            for j in start..end {
                if i != j {
                    let ins = if words[i] < words[j] {
                        (words[i].clone(), words[j].clone())
                    } else {
                        (words[j].clone(), words[i].clone())
                    };
                    *co_occurrence.entry(ins).or_insert(0.0) += 1.0;
                }
            }
        }
        
        let mut vocab: HashMap<String, usize> = HashMap::new();
        for word in &words {
            if !vocab.contains_key(word) {
                vocab.insert(word.clone(), vocab.len());
            }
        }
        let mut embedding = vec![0.0; vocab.len().max(vdim)];
        
        for (w, &i) in &vocab {
            if i < embedding.len() {
                let mut score = 0.0;
                for ((w1, w2), count) in &co_occurrence {
                    if w1 == w || w2 == w {
                        score += count;
                    }
                }
                embedding[i] = score / co_occurrence.len().max(1) as f32;
            }
        }
        
        normalize(&mut embedding);
        pad(&mut embedding, vdim);
        embedding
    }

    /// TF-IDF embedding with document frequency normalization
    pub fn tfidf_embedding(text: &str,vdim:usize) -> Vec<f32> {
        let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
        let words: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_lowercase()).collect();
        if words.is_empty() {
            return vec![0.0; vdim];
        }
        
        let mut vocab: HashMap<String, usize> = HashMap::new();
        let mut tf: HashMap<String, f32> = HashMap::new();
        for word in &words {
            if !vocab.contains_key(word) {
                vocab.insert(word.clone(), vocab.len());
            }
            *tf.entry(word.clone()).or_insert(0.0) += 1.0;
        }
        
        let (tot,tot_uniq) = (words.len() as f32,vocab.len() as f32);
        let mut embedding = vec![0.0; vocab.len().max(vdim)];
        
        for (word, count) in tf {
            if let Some(&idx) = vocab.get(&word) {
                if idx < embedding.len() {
                    let tf = count / tot;
                    let idf = (tot_uniq / (1.0 + count)).ln_1p();
                    embedding[idx] = tf * idf;
                }
            }
        }
        
        normalize(&mut embedding);
        pad(&mut embedding, vdim);   
        embedding
    }

    /// Uses Word adjacency graph
    pub fn graph_embedding(text: &str,vdim:usize) -> Vec<f32> {
        let re = Regex::new(r"[a-zA-Z0-9]+").unwrap();
        let words: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_lowercase()).collect();
        // Need two nodes for an edge 
        if words.len() < 2 {
            return vec![0.0; vdim];
        }
        
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        for i in 0..words.len() - 1 {
            adjacency.entry(words[i].clone()).or_insert_with(HashSet::new).insert(words[i + 1].clone());
            adjacency.entry(words[i + 1].clone()).or_insert_with(HashSet::new).insert(words[i].clone());
        }
        
        // Calculate graph centrality measures
        let mut vocab: HashMap<String, usize> = HashMap::new();
        for word in &words {
            if !vocab.contains_key(word) {
                vocab.insert(word.clone(), vocab.len());
            }
        }
        let mut embedding = vec![0.0; vocab.len().max(vdim)];
        
        for (word, &idx) in &vocab {
            if idx < embedding.len() {
                // Degree centrality
                let degree = adjacency.get(word).map(|s| s.len()).unwrap_or(0) as f32;
                // PageRank-like score
                let mut pagerank = degree;
                if let Some(neighbors) = adjacency.get(word) {
                    for neighbor in neighbors {
                        if let Some(neighbor_degree) = adjacency.get(neighbor).map(|s| s.len()) {
                            pagerank += 1.0 / (neighbor_degree as f32 + 1.0);
                        }
                    }
                }
                
                embedding[idx] = pagerank / (words.len() as f32);
            }
        }
        
        normalize(&mut embedding);
        pad(&mut embedding, vdim);
        embedding
    }

    fn normalize(vec: &mut [f32]) {
        let mag: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag > 0.0 {
            for val in vec.iter_mut() {
                *val /= mag;
            }
        }
    }

    fn pad(vec: &mut Vec<f32>, target_dim: usize) {
        vec.resize(target_dim, 0.0);
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        let dot_prod: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mod_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mod_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mod_a == 0.0 || mod_b == 0.0 {
            0.0
        } else {
            dot_prod / (mod_b * mod_a)
        }
    }

    pub fn hybrid_embedding(text: &str, weights: &[(EmbedMethod, f32)],vdim:usize,win_len:Option<usize>) -> Vec<f32> {
        let mut ret = vec![0.0; vdim];
        let tot_wt: f32 = weights.iter().map(|(_, w)| w).sum();
        if tot_wt == 0.0 {
            return ret;
        }
        for (m, w) in weights {
            let embedding = generate_embedding(text, *m,vdim,win_len.unwrap_or(3));
            let normalized_weight = w / tot_wt;
            
            for (i, &val) in embedding.iter().enumerate() {
                ret[i] += val * normalized_weight;
            }
        }
        normalize(&mut ret);
       ret 
    }
}
