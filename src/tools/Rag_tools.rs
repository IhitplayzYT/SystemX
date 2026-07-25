pub mod rag_tools{
    use std::collections::HashMap;

use crate::{helper::Helper::CLI, tools::tools::Tools::Tool, vstore::{embed::embed::EmbedMethod, vstore::Vstore::VectorStore}};
    use crate::vstore::embed::embed::generate_embedding;
    use anyhow::anyhow;

    fn retrive(query:String,vstore: VectorStore,method:EmbedMethod,vdim:usize,win_len:usize,top_k:usize) -> String{
        vstore.get_top_k(&generate_embedding(&query[..], method, vdim, win_len),top_k).unwrap().iter().map(|x| format!("{} Score:{}",vstore.get_text(&x.id).unwrap().unwrap(),x.score)).fold("".to_string(), |acc,x| acc+&x+"\n").trim().to_string()
    }


    pub struct Get_top_k{
        pub embed_model:EmbedMethod,
        pub top_k:usize,
        pub vdim:usize,
        pub win_len:usize,
    }

    impl Get_top_k{
        pub fn new(embed_model:EmbedMethod,top_k:usize,vdim:usize,win_len:usize) -> Self{
            Self { embed_model, top_k, vdim, win_len }
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
            ctx:&mut crate::tools::tools::Tools::AgentContext,
            args: serde_json::Value,
        ) -> anyhow::Result<String>
        {

            
        }


    }




    pub struct Filter_by_metadata{
        pub embed_model:EmbedMethod,
        pub top_k:usize,
        pub vdim:usize,
        pub win_len:usize,
    }

    pub fn filter_meta(query:String,meta:HashMap<String,String>,vstore: VectorStore,method:EmbedMethod,vdim:usize,win_len:usize,top_k:usize) -> String{
        vstore.search_with_metadata_filter(&generate_embedding(&query, method, vdim, win_len), top_k,meta);
    }



}