use candle::Device;
use tokio::time::Instant;
use tracing::info;
use crate::embed::embedder::CandleEmbedBuilder;
use crate::embed::models::Model;

mod embed;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let text = "
    1/30/2024: Release BGE-M3, a new member to BGE model series! M3 stands for Multi-linguality (100+ languages), Multi-granularities (input length up to 8192), Multi-Functionality (unification of dense, lexical, multi-vec/colbert retrieval). It is the first embedding model that supports all three retrieval methods, achieving new SOTA on multi-lingual (MIRACL) and cross-lingual (MKQA) benchmarks. Technical Report and Code. :fire:
    1/9/2024: Release Activation-Beacon, an effective, efficient, compatible, and low-cost (training) method to extend the context length of LLM. Technical Report :fire:
    12/24/2023: Release LLaRA, a LLaMA-7B based dense retriever, leading to state-of-the-art performances on MS MARCO and BEIR. Model and code will be open-sourced. Please stay tuned. Technical Report :fire:
    11/23/2023: Release LM-Cocktail, a method to maintain general capabilities during fine-tuning by merging multiple language models. Technical Report :fire:
    10/12/2023: Release LLM-Embedder, a unified embedding model to support diverse retrieval augmentation needs for LLMs. Technical Report
    09/15/2023: The technical report and massive training data of BGE has been released
    09/12/2023: New models:
        New reranker model: release cross-encoder models BAAI/bge-reranker-base and BAAI/bge-reranker-large, which are more powerful than embedding model. We recommend to use/fine-tune them to re-rank top-k documents returned by embedding models.
        update embedding model: release bge-*-v1.5 embedding model to alleviate the issue of the similarity distribution, and enhance its retrieval ability without instruction.

    1/30/2024: Release BGE-M3, a new member to BGE model series! M3 stands for Multi-linguality (100+ languages), Multi-granularities (input length up to 8192), Multi-Functionality (unification of dense, lexical, multi-vec/colbert retrieval). It is the first embedding model that supports all three retrieval methods, achieving new SOTA on multi-lingual (MIRACL) and cross-lingual (MKQA) benchmarks. Technical Report and Code. :fire:
    1/9/2024: Release Activation-Beacon, an effective, efficient, compatible, and low-cost (training) method to extend the context length of LLM. Technical Report :fire:
    12/24/2023: Release LLaRA, a LLaMA-7B based dense retriever, leading to state-of-the-art performances on MS MARCO and BEIR. Model and code will be open-sourced. Please stay tuned. Technical Report :fire:
    11/23/2023: Release LM-Cocktail, a method to maintain general capabilities during fine-tuning by merging multiple language models. Technical Report :fire:
    10/12/2023: Release LLM-Embedder, a unified embedding model to support diverse retrieval augmentation needs for LLMs. Technical Report
    09/15/2023: The technical report and massive training data of BGE has been released
    09/12/2023: New models:
        New reranker model: release cross-encoder models BAAI/bge-reranker-base and BAAI/bge-reranker-large, which are more powerful than embedding model. We recommend to use/fine-tune them to re-rank top-k documents returned by embedding models.
        update embedding model: release bge-*-v1.5 embedding model to alleviate the issue of the similarity distribution, and enhance its retrieval ability without instruction.

";

    // let api = hf_hub::api::sync::Api::new().unwrap();
    // let repo = api
    //     .model("WhereIsAI/UAE-Large-V1".to_string())
    //     .get("tokenizer.json")
    //     .unwrap();
    // let mut builder = TokenBuilder::default();
    // builder.tokenizer_file = repo;
    // builder.input_size = 512;
    // builder.overlap = (10 / 100 * 512) as u32;
    // builder.special_tokens = true;
    //
    // let tokenize = Tokenize::new(builder).unwrap();
    // let encoding = tokenize.encode(text.to_string()).unwrap();
    // println!("{:?}", encoding.get_tokens());
    // println!("{}", encoding.get_tokens().len());
    // println!("{}", text.split_whitespace().collect::<Vec<_>>().len());
    //
    // let encodings = tokenize.split_tokens(text.to_string()).unwrap();
    // println!("{:?}", encodings);
    // println!("{}", encodings.len());
    // println!("{}", encodings.first().unwrap().len());

    let mut candle = CandleEmbedBuilder::new()
        .padding(true)
        .model(Model::UaeLargeV1)
        .device(Device::new_cuda(0).unwrap())
        .build()
        .await.unwrap();
    
    info!("embedding start");
    let now = Instant::now();
    let embeddings = candle.split_embed(text, true).unwrap();
    info!("embedded in {}", now.elapsed().as_secs_f32());
    
    // println!("{:#?}", embeddings);
    for i in embeddings {
        println!("{}", i.len());
    }
    
    let txt = "hello how are you";
    let embedding = candle.embed(txt, true, true).unwrap();
    // println!("{:#?}", embedding);
    for i in embedding {
        println!("{}", i.len());
    }
}
