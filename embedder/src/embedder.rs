use anyhow::{Error, Result};
use candle::Device;
use itertools::Itertools;
use pgvector::Vector;
use sqlx::Acquire;
use tokio::time::Instant;
use tracing::{error, info};
use ulid::Ulid;

use db::Db;
use utils::amqprs::channel::{BasicAckArguments, Channel};
use utils::amqprs::consumer::AsyncConsumer;
use utils::amqprs::{BasicProperties, Deliver};
use utils::async_trait::async_trait;

use crate::embed::candle_embed::{CandleEmbed, CandleEmbedBuilder};
use crate::embed::models::Model;

pub struct Embedder {
    db: Db,
    embed: CandleEmbed,
    pub auto_ack: bool,
}

impl Embedder {
    pub async fn new() -> Result<Self> {
        let db = Db::new(5).await?;

        let embed = CandleEmbedBuilder::new()
            .padding(true)
            .model(Model::UaeLargeV1)
            .mean_pooling(false)
            .device(Device::new_cuda(0).unwrap())
            .build()
            .await?;

        Ok(Self {
            db,
            embed,
            auto_ack: true,
        })
    }

    async fn get_document(&self, id: &str) -> Result<Option<String>> {
        let mut pool = self.db.get_pg().await?;

        let res = sqlx::query!(
            r#"
            SELECT content FROM document WHERE doc_id=$1
            "#,
            id
        )
        .fetch_one(pool.acquire().await?)
        .await?;

        Ok(res.content)
    }

    async fn save_embeddings(
        &self,
        embeddings: Vec<(Vec<f32>, (usize, usize))>,
        doc_id: &str,
    ) -> Result<()> {
        let embedding = embeddings
            .iter()
            .map(|e| Vector::from(e.0.clone()))
            .collect::<Vec<_>>();

        let (chunk_starts, chunk_ends): (Vec<_>, Vec<_>) = embeddings
            .iter()
            .map(|e| (e.1 .0 as i64, e.1 .1 as i64))
            .unzip();

        let chunk_ids = (0..embedding.len())
            .map(|_| Ulid::new().to_string())
            .collect_vec();

        let doc_ids = vec![doc_id.to_string(); embedding.len()];

        let mut pool = self.db.get_pg().await?;

        let res = sqlx::query!(
            r#"
                INSERT INTO chunk (chunk_id, doc_id, chunk_start, chunk_end, embedding)
                SELECT * FROM UNNEST(
                    $1::text[],
                    $2::text[],
                    $3::bigint[],
                    $4::bigint[],
                    $5::vector[]
                    )
            "#,
            &chunk_ids,
            &doc_ids,
            &chunk_starts,
            &chunk_ends,
            embedding as Vec<Vector>
        )
        .execute(pool.acquire().await?)
        .await?;

        info!(
            "wrote {} chunks to db, rows affected {}",
            chunk_ids.len(),
            res.rows_affected()
        );

        Ok(())
    }

    pub async fn embedder(&mut self, id: &str) -> Result<()> {
        let document = self.get_document(id).await?;

        if document.is_none() {
            return Err(Error::msg(format!("no document found in db for {id}")));
        }

        let embeddings = self.embed.split_embed(&document.unwrap(), true)?;
        self.save_embeddings(embeddings, &id).await?;

        Ok(())
    }
}

#[async_trait]
impl AsyncConsumer for Embedder {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // ack explicitly if manual ack
        if !self.auto_ack {
            info!("ack to delivery {} on channel {}", deliver, channel);
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).await.unwrap();
        }

        let id = String::from_utf8(content).unwrap();
        info!("received id from parser, embedding now {id}");
        let now = Instant::now();
        if let Err(e) = self.embedder(&id).await {
            error!("amq consumer::embedder error while embedding id {id}: {e}");
        }
        info!("embedded {id} in {}", now.elapsed().as_secs_f32());
    }
}
