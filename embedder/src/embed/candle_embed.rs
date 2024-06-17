use crate::embed::models::Model;
use anyhow::{anyhow, Error, Result};
use candle::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, HiddenAct, DTYPE};
use hf_hub::{api::tokio::Api, Repo, RepoType};
use tokenizers::{Encoding, PaddingParams, Tokenizer, TruncationParams};
use tokio::time::Instant;
use tracing::info;

#[derive(Debug, Clone)]
pub struct CandleEmbedBuilder {
    pub approximate_gelu: bool,
    pub model: Model,
    pub mean_pooling: bool,
    pub normalize: bool,
    pub device: Device,
    pub padding: bool,
    pub overlap: usize,
}

impl Default for CandleEmbedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CandleEmbedBuilder {
    pub fn new() -> Self {
        CandleEmbedBuilder {
            approximate_gelu: false,
            model: Model::UaeLargeV1,
            mean_pooling: true,
            normalize: false,
            device: Device::Cpu,
            padding: false,
            overlap: 52,
        }
    }

    #[allow(dead_code)]
    pub fn approximate_gelu(mut self, val: bool) -> Self {
        self.approximate_gelu = val;
        self
    }

    pub fn model(mut self, model: Model) -> Self {
        self.model = model;
        self
    }

    pub fn mean_pooling(mut self, pool: bool) -> Self {
        self.mean_pooling = pool;
        self
    }

    #[allow(dead_code)]
    pub fn normalize(mut self, norm: bool) -> Self {
        self.normalize = norm;
        self
    }

    pub fn padding(mut self, pad: bool) -> Self {
        self.padding = pad;
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.device = device;
        self
    }

    pub async fn build(self) -> Result<CandleEmbed> {
        let model_id = self.model.to_string();
        let repo = Repo::new(model_id, RepoType::Model);

        info!("downloading config, tokenizer and weights file");
        let (config_file, tokenizer_file, weights) = {
            let api = Api::new()?.repo(repo);
            let config = api.get("config.json").await?;
            let tokenizer = api.get("tokenizer.json").await?;
            let weights = api.get("model.safetensors").await?;
            (config, tokenizer, weights)
        };

        println!("{config_file:?}, {tokenizer_file:?}, {weights:?}");

        let config_data = tokio::fs::read_to_string(config_file).await?;
        let config_json: serde_json::Value = serde_json::from_str(&config_data)?;

        let hidden_size = config_json
            .get("hidden_size")
            .ok_or_else(|| anyhow::Error::msg("hidden_size field missing"))?
            .as_u64()
            .ok_or_else(|| anyhow::Error::msg("hidden_size is not a u64"))?
            as usize;
        let max_position_embeddings = config_json
            .get("max_position_embeddings")
            .ok_or_else(|| anyhow::Error::msg("max_position_embeddings field missing"))?
            .as_u64()
            .ok_or_else(|| anyhow::Error::msg("max_position_embeddings is not a u64"))?
            as usize;

        info!("config loaded");
        let mut config: Config = serde_json::from_value(config_json)?;
        if self.approximate_gelu {
            config.hidden_act = HiddenAct::GeluApproximate;
        }

        let tokenizer = Tokenizer::from_file(tokenizer_file).map_err(Error::msg)?;
        info!("loaded tokenizer");
        let now = Instant::now();
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights], DTYPE, &self.device)? };
        let model = BertModel::load(vb, &config)?;

        info!(
            "model loaded in: {}, Device: {:?}, overlap: {}, normalize: {}",
            now.elapsed().as_secs_f32(),
            self.device,
            self.overlap,
            self.normalize
        );

        Ok(CandleEmbed {
            config,
            mean_pooling: self.mean_pooling,
            model,
            tokenizer,
            normalize: self.normalize,
            model_dim: hidden_size,
            model_input_size: max_position_embeddings,
            model_id: self.model,
            padding: self.padding,
            overlap: self.overlap,
        })
    }
}

#[allow(dead_code)]
pub struct CandleEmbed {
    config: Config,
    mean_pooling: bool,
    model: BertModel,
    tokenizer: Tokenizer,
    normalize: bool,
    pub model_dim: usize,
    pub model_id: Model,
    pub model_input_size: usize,
    padding: bool,
    overlap: usize,
}

#[allow(dead_code)]
impl CandleEmbed {
    pub fn token_count(&mut self, text: &str, add_special: bool) -> Result<usize> {
        let encoding = self.tokenize(text, false, add_special)?;
        Ok(encoding.get_tokens().len())
    }

    fn get_tokenizer(&mut self, truncate: bool) -> Result<Tokenizer> {
        let tokenizer = {
            let mut trunc = None;
            if truncate {
                trunc = Some(TruncationParams {
                    max_length: self.model_input_size,
                    ..Default::default()
                });
            }
            let mut padding = None;
            if self.padding {
                padding = Some(PaddingParams::default());
            }

            &self
                .tokenizer
                .with_padding(padding)
                .with_truncation(trunc)
                .map_err(Error::msg)?
                .clone()
        };

        Ok(tokenizer.clone().into())
    }
    pub fn tokenize(&mut self, text: &str, truncate: bool, add_special: bool) -> Result<Encoding> {
        let tokenizer = self.get_tokenizer(truncate)?;
        tokenizer.encode(text, add_special).map_err(Error::msg)
    }

    #[allow(dead_code)]
    pub fn split_tokenize(
        &mut self,
        _text: String,
        _truncate: bool,
        _add_special: bool,
    ) -> Result<Vec<Encoding>> {
        todo!()
    }

    pub fn embed(&mut self, text: &str, truncate: bool, add_special: bool) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(Error::msg(
                "CandleEmbed error: embed called with empty text",
            ));
        }

        if !truncate {
            let count = self.token_count(text, add_special)?;
            if count > self.model_input_size {
                return Err(anyhow!(format!(
                    "expected {} tokens, got {} tokens",
                    self.model_input_size, count
                )));
            }
        }

        let encoding = self.tokenize(text, truncate, add_special)?;
        let device = &self.model.device;
        let tokens = encoding.get_ids().to_vec();
        let token_ids = Tensor::new(&tokens[..], device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;

        let outputs = self.model.forward(&token_ids, &token_type_ids)?;

        let embedding = if self.mean_pooling {
            // Mean pooling
            let (_n_sentence, n_tokens, _hidden_size) = outputs.dims3()?;
            (outputs.sum(1)? / (n_tokens as f64))?
        } else {
            // CLS only
            outputs.i((.., 0))?
        };

        let embedding = if self.normalize {
            embedding.broadcast_div(&embedding.sqr()?.sum_keepdim(1)?.sqrt()?)?
        } else {
            embedding
        };

        Ok(embedding.i(0)?.to_vec1::<f32>()?)
    }

    pub fn embed_batch(
        &mut self,
        texts: &[&str],
        truncate: bool,
        add_special: bool,
    ) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Err(Error::msg(
                "CandleEmbed error: embed_batch called with empty texts",
            ));
        }
        let mut embeddings = vec![];
        for text in texts {
            let embedding = self.embed(text, truncate, add_special)?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    #[allow(clippy::type_complexity)]
    pub fn split_embed(
        &mut self,
        text: &str,
        add_special: bool,
    ) -> Result<Vec<(Vec<f32>, (usize, usize))>> {
        if text.is_empty() {
            return Err(Error::msg("split_embed: text is empty"));
        };

        let encoding = self.tokenize(text, false, add_special)?;
        let token_ids = encoding.get_ids();
        let offsets = encoding.get_offsets();

        let chunk_size = self.model_input_size - self.overlap;

        let mut cur_len = 0;
        let mut embeddings = vec![];

        for ids in token_ids.chunks(chunk_size) {
            cur_len += ids.len();
            let chunk_start = cur_len - ids.len();
            let chunk_end = cur_len;

            let mut overlapped: &[u32] = &[];
            if (cur_len + self.overlap) < token_ids.len() {
                overlapped = &token_ids[cur_len..(cur_len + self.overlap)];
            }
            let mut ids = ids.to_vec();
            ids.extend_from_slice(overlapped);

            info!("embedding {} tokens", ids.len());
            let token_ids = Tensor::new(ids.as_slice(), &self.model.device)?.unsqueeze(0)?;
            let token_type_ids = token_ids.zeros_like()?;

            let outputs = self.model.forward(&token_ids, &token_type_ids)?;

            let embedding = if self.mean_pooling {
                // Mean pooling
                let (_n_sentence, n_tokens, _hidden_size) = outputs.dims3()?;
                (outputs.sum(1)? / (n_tokens as f64))?
            } else {
                // CLS only
                outputs.i((.., 0))?
            };

            let embedding = if self.normalize {
                embedding.broadcast_div(&embedding.sqr()?.sum_keepdim(1)?.sqrt()?)?
            } else {
                embedding
            };

            let offsets = (offsets[chunk_start].0, offsets[chunk_end - 2].1);

            embeddings.push((embedding.i(0)?.to_vec1::<f32>()?, offsets));
        }

        Ok(embeddings)
    }
}
