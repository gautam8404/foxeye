use anyhow::{anyhow, Result};
use pgvector::Vector;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, FromRow};
use std::collections::HashSet;
use std::iter::Iterator;
use std::string::ToString;
use tracing::error;

use db::Db;
use embedder::models::Model;
use embedder::Device;
use embedder::{CandleEmbed, CandleEmbedBuilder};
use crate::misc::STOPWORDS;


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
                WITH ranked_chunks AS (
                    SELECT
                        chunk_id,
                        chunk.doc_id,
                        chunk_start,
                        chunk_end,
                        embedding,
                        1 - (embedding <=> $1) AS cosine_similarity,
                        ROW_NUMBER() OVER (PARTITION BY chunk.doc_id ORDER BY embedding <=> $1) AS rank
                    FROM
                        chunk
                )
                SELECT
                    rc.chunk_id,
                    rc.chunk_start,
                    rc.chunk_end,
                    rc.cosine_similarity,
                    d.url,
                    d.content,
                    d.title
                FROM
                    ranked_chunks rc
                JOIN
                    document d ON rc.doc_id = d.doc_id
                WHERE
                    rc.rank = 1
                ORDER BY
                    rc.cosine_similarity DESC
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

    fn summarise(&mut self, text: &str, query: &str, min_window: usize) -> Result<String> {
        let reg = Regex::new(r"\[.*?]|[^\x00-\x7F]+| {4}|[\t\n\r]")?;
        let text = reg.replace_all(&text, "").to_string().to_lowercase();
        let query = query.to_lowercase();
        let text_vec = text.split_whitespace().collect::<Vec<_>>();
        
        if text_vec.len() < min_window + 1 {
            return Err(anyhow!("document too small"));
        }

        let mut keyword_location = vec![];
        let mut last = 0;
        for (i, word) in text_vec.iter().enumerate() {
            if !STOPWORDS.contains(*word) && query.contains(*word) {
                if last == 0 {
                    keyword_location.push((0, i));
                    last = i;
                    continue;
                }
                keyword_location.push((i - last, i));
                last = i;
            }
        }
        
        if keyword_location.is_empty() {
            return Err(anyhow!("failed to summarise"));
        }

        keyword_location.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap()
        });

        let start = keyword_location[0].1;
        let mut end = 0;
        
        for (_a,b) in keyword_location {
            if (b - start) > min_window {
                end = b;
            }
        }

        if start < end && end < text_vec.len() {
            Ok(text_vec[start..end].join(" "))
        } else {
            Err(anyhow!("failed to summarise"))
        }
    }

    pub async fn search(&mut self, input: SearchInput) -> Result<Vec<SearchResult>> {
        let (query, limit, offset) = (input.query, input.limit, input.offset);

        let embedding = self.embed_query(query.clone())?;
        let chunks = self.get_documents(embedding, limit, offset).await?;

        let mut res = vec![];

        for chunk in chunks {
            if chunk.url.is_some() {
                let chunk_id = chunk.chunk_id.unwrap();
                let title = chunk.title.unwrap_or("".to_string());
                let content = chunk.content.unwrap_or("".to_string());
                let url = chunk.url.unwrap();
                let score = chunk.cosine_similarity.unwrap();
                let chunk_start = chunk.chunk_start.unwrap_or(0) as usize;
                let mut chunk_end = chunk.chunk_end.unwrap_or((content.len() / 4) as i64) as usize;
                if chunk_start > chunk_end {
                    chunk_end = content.len() - chunk_start;
                }

                let content = content.chars().collect::<Vec<_>>();

                if chunk_start > chunk_end
                    || chunk_end > content.len()
                    || chunk_start > content.len()
                {
                    res.push(SearchResult {
                        url: url.clone(),
                        score,
                        summary: "".to_string(),
                        title,
                    });
                    continue;
                }

                let summary = &content[chunk_start..chunk_end];
                let summary = summary.iter().collect::<String>();
                let q = format!("{query} {title}");
                let summary = self
                    .summarise(&summary, &q, 100)
                    .map_err(|e| {
                        error!("failed to summarise chunk {chunk_id}: {e}");
                        e
                    })
                    .unwrap_or(summary);

                res.push(SearchResult {
                    url,
                    score,
                    summary,
                    title,
                })
            }
        }

        Ok(res)
    }
}

#[derive(Debug, FromRow)]
pub struct Chunk {
    pub chunk_id: Option<String>,
    pub chunk_start: Option<i64>,
    pub chunk_end: Option<i64>,
    pub cosine_similarity: Option<f64>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub score: f64,
    pub summary: String,
    pub title: String,
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
        1 - (embedding <=> $1) AS cosine_similarity,
        ROW_NUMBER() OVER (PARTITION BY chunk.doc_id ORDER BY embedding <=> $1) AS rank
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
LIMIT $2 OFFSET $3;
 */
