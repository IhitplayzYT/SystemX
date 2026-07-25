pub mod rag_tools{
    use std::collections::HashMap;
    use crate::{helper::Helper::{COLLECTION_N, VSTORE_N}, tools::tools::Tools::Tool, vstore::{embed::embed::EmbedMethod, vstore::Vstore::{VectorStore, VectorStoreConfig}}};
    use crate::vstore::embed::embed::generate_embedding;
    use anyhow::anyhow;

    fn retrieve(query:String, vstore: &VectorStore, method:EmbedMethod, vdim:usize, win_len:usize, top_k:usize) -> anyhow::Result<String>{
        let results = vstore.get_top_k(&generate_embedding(&query, method, vdim, win_len), top_k).map_err(|e| anyhow!("Failed to get top k: {}", e))?;
        Ok(results.iter().map(|x| {
                let text = vstore.get_text(&x.id).ok().flatten().unwrap_or_else(|| "No text available".to_string());
                format!("{} \nScore: {:.4}\nID: {}", text, x.score, x.id)
            }).collect::<Vec<String>>().join("\n\n---\n\n"))
    }

    fn filter_meta(query:String, meta:HashMap<String,String>, vstore: &VectorStore, method:EmbedMethod, vdim:usize, win_len:usize, top_k:usize) -> anyhow::Result<String>{
        let results = vstore.search_with_metadata_filter(&generate_embedding(&query, method, vdim, win_len), top_k, meta).map_err(|e| anyhow!("Failed to search with metadata filter: {}", e))?;
        Ok(results.iter().map(|x| {
                let text = vstore.get_text(&x.id).ok().flatten().unwrap_or_else(|| "No text available".to_string());
                let metadata = vstore.get_metadata(&x.id).ok().flatten().unwrap_or_else(|| HashMap::new());                
                let meta_str = metadata.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
                format!("{} \nScore: {:.4}\nMetadata: {{{}}}\nID: {}", text, x.score, meta_str, x.id)
            }).collect::<Vec<String>>().join("\n\n---\n\n"))
    }


    pub struct Get_top_k{
        pub vstore: Option<VectorStore>,
        pub embed_model:EmbedMethod,
        pub top_k:usize,
        pub vdim:usize,
        pub win_len:usize,
    }

    impl Get_top_k{
        pub fn new(embed_model:EmbedMethod,top_k:usize,vdim:usize,win_len:usize) -> Self{

            Self { embed_model, top_k, vdim, win_len,vstore:None}
            }

        pub fn add_vstore(&mut self,vstorepath:String,collection:String){
            let vstore_conf = VectorStoreConfig{path:vstorepath,collection_name:collection};
            let vstore = VectorStore::new(vstore_conf).unwrap();
            self.vstore = Some(vstore)
        }

        }

    impl Tool for Get_top_k{
        fn description(&self) -> &'static str {
            "Returns the top k closest vector embedded chunks to the query"
        }
        fn name(&self) -> &'static str {
            "get_top_k"
        }
        fn execute(
            &self,
            _ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'query' argument"))?
                .to_string();
            let vstore = if let Some(x) = &self.vstore{
                x
            }else{
                &VectorStore::default()
            };

            retrieve(query,vstore, self.embed_model, self.vdim, self.win_len, self.top_k)
        }
    }




    pub struct Filter_by_metadata{
        pub vstore: Option<VectorStore>,
        pub embed_model:EmbedMethod,
        pub top_k:usize,
        pub vdim:usize,
        pub win_len:usize,
    }

    impl Filter_by_metadata{
        pub fn new(embed_model:EmbedMethod,top_k:usize,vdim:usize,win_len:usize) -> Self{
            Self { embed_model, top_k, vdim, win_len,vstore:None}
        }

        pub fn add_vstore(&mut self,vstorepath:String,collection:String){
            let vstore_conf = VectorStoreConfig{path:vstorepath,collection_name:collection};
            let vstore = VectorStore::new(vstore_conf).unwrap();
            self.vstore = Some(vstore)
        }

    }

    impl Tool for Filter_by_metadata{
        fn description(&self) -> &'static str {
            "Returns the top k closest vector embedded chunks to the query filtered by metadata"
        }
        fn name(&self) -> &'static str {
            "filter_by_metadata"
        }
        fn execute(
            &self,
            _ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'query' argument"))?
                .to_string();
            
            let metadata_value = args.get("metadata")
                .ok_or_else(|| anyhow!("Missing 'metadata' argument"))?;
            
            let metadata: HashMap<String, String> = serde_json::from_value(metadata_value.clone())
                .map_err(|e| anyhow!("Invalid metadata format: {}", e))?;
            let vstore = if let Some(x) = &self.vstore{
                x
            }else{
                &VectorStore::default()
            };
            filter_meta(query, metadata, vstore, self.embed_model, self.vdim, self.win_len, self.top_k)
        }
    }

    // Create instances for easy registration
    pub fn get_top_k_default() -> Get_top_k {
        Get_top_k::new(EmbedMethod::Semantic, 5, 384, 3)
    }

    pub fn filter_by_metadata_default() -> Filter_by_metadata {
        Filter_by_metadata::new(EmbedMethod::Semantic, 5, 384, 3)
    }
}