use anyhow::Result;
use pgvector::Vector;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, FromRow};
use tracing::error;

use db::Db;
use embedder::models::Model;
use embedder::Device;
use embedder::{CandleEmbed, CandleEmbedBuilder};

pub struct Searcher {
    db: Db,
    embed: CandleEmbed,
}

impl Searcher {
    pub async fn new() -> Result<Self> {
        let db = Db::new(5).await?;
        let embed = CandleEmbedBuilder::new()
            .padding(true)
            .model(Model::UaeLargeV1)
            .mean_pooling(false)
            .device(Device::new_cuda(0).unwrap())
            .build()
            .await?;

        Ok(Searcher { db, embed })
    }

    fn embed_query(&mut self, query: String) -> Result<Vector> {
        let embed = self.embed.embed(&query, true, true)?;
        let embedding = Vector::from(embed);

        Ok(embedding)
    }

    async fn get_documents(
        &self,
        embedding: Vector,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Chunk>> {
        let mut pool = self.db.get_pg().await?;
        let res = sqlx::query_as!(
            Chunk,
            r#"
                SELECT
                    chunk_id,
                    chunk_start,
                    chunk_end,
                    1 - (embedding <=> $1) AS cosine_similarity,
                    d.url,
                    d.content
                FROM
                    chunk
                JOIN
                    document d ON chunk.doc_id = d.doc_id
                ORDER BY
                    embedding <=> $1
                LIMIT $2 OFFSET $3;
            "#,
            embedding as Vector,
            limit as i32,
            offset as i32
        )
        .fetch_all(pool.acquire().await?)
        .await?;

        Ok(res)
    }

    fn summarise(&mut self, text: &str) -> Result<String> {
        // will do something more here

        let reg = Regex::new(r"\[.*?]|[^\x00-\x7F]+| {4}|[\t\n\r]")?;
        let text = reg.replace_all(&text, "").to_string();

       Ok(text)
    }

    pub async fn search(&mut self, input: SearchInput) -> Result<Vec<SearchResult>> {
        let (query, limit, offset) = (input.query, input.limit, input.offset);

        let embedding = self.embed_query(query)?;
        let chunks = self.get_documents(embedding, limit, offset).await?;

        let mut res = vec![];

        for chunk in chunks {
            if chunk.url.is_some() {
                let chunk_id = chunk.chunk_id.unwrap();
                let content = chunk.content.unwrap_or("".to_string());
                let url = chunk.url.unwrap();
                let score = chunk.cosine_similarity.unwrap();
                let chunk_start = chunk.chunk_start.unwrap_or(0) as usize;
                let mut chunk_end = chunk.chunk_end.unwrap_or((content.len() / 4) as i64) as usize;
                if chunk_start > chunk_end {
                    chunk_end = content.len() - chunk_start;
                }

                if chunk_start > chunk_end {
                    res.push(SearchResult {
                        url: url.clone(),
                        score,
                        summary: "".to_string(),
                    })
                }

                let content = content.chars().collect::<Vec<_>>();
                let summary = &content[chunk_start..chunk_end];
                let summary = summary.iter().collect::<String>();
                let summary = self.summarise(&summary).map_err(|e| {
                    error!("failed to summarise chunk {chunk_id}: {e}");
                    e
                }).unwrap_or(summary);

                res.push(SearchResult {
                    url,
                    score,
                    summary,
                })
            }
        }

        Ok(res)
    }
}

fn cosine_similarity(vec1: &Vec<f32>, vec2: &Vec<f32>) -> f32 {
    let dot_product = vec1
        .iter()
        .zip(vec2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();
    let magnitude1 = vec1.iter().map(|v| v * v).sum::<f32>().sqrt();
    let magnitude2 = vec2.iter().map(|v| v * v).sum::<f32>().sqrt();
    dot_product / (magnitude1 * magnitude2)
}

#[derive(Debug, FromRow)]
pub struct Chunk {
    pub chunk_id: Option<String>,
    pub chunk_start: Option<i64>,
    pub chunk_end: Option<i64>,
    pub cosine_similarity: Option<f64>,
    pub url: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    query: String,
    limit: u32,
    offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub score: f64,
    pub summary: String,
}

/*
alternate query

WITH ranked_chunks AS (
    SELECT
        chunk_id,
        chunk.doc_id,
        chunk_start,
        chunk_end,
        embedding,
        chunk.created_at,
        chunk.updated_at,
        1 - (embedding <=> '[1.0, 0.5, ..., 0.2]') AS cosine_similarity,
        ROW_NUMBER() OVER (PARTITION BY chunk.doc_id ORDER BY embedding <=> '[1.0, 0.5, ..., 0.2]') AS rank
    FROM
        chunk
)
SELECT
    rc.chunk_id,
    rc.doc_id,
    rc.chunk_start,
    rc.chunk_end,
    rc.embedding,
    rc.created_at,
    rc.updated_at,
    rc.cosine_similarity,
    d.url,
    d.content
FROM
    ranked_chunks rc
JOIN
    document d ON rc.doc_id = d.doc_id
WHERE
    rc.rank = 1
ORDER BY
    rc.cosine_similarity DESC
LIMIT 10;
 */
