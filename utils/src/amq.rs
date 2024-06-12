use std::fmt::{Debug, Formatter};

use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{
    BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, Channel,
    ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments,
};
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

#[derive(Clone)]
pub struct RabbitMQ {
    pub channel: Channel,
    #[allow(dead_code)]
    pub connection: Connection,
    queue: String,
    routing_key: String,
    exchange_name: String,
    pub consumer_tag: String,
}

impl Debug for RabbitMQ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RabbitMQ")
            .field("queue", &self.queue)
            .field("routing_key", &self.routing_key)
            .field("exchange_name", &self.exchange_name)
            .field("consumer_tag", &self.consumer_tag)
            .finish()
    }
}

impl RabbitMQ {
    pub async fn new(
        uri: &str,
        queue: &str,
        consumer_tag: &str,
        routing_key: &str,
        exchange_name: &str,
    ) -> Result<RabbitMQ> {
        let args: OpenConnectionArguments = uri.try_into().unwrap();
        let connection = Connection::open(&args).await?;

        connection
            .register_callback(DefaultConnectionCallback)
            .await?;

        let channel = connection.open_channel(None).await?;

        channel.register_callback(DefaultChannelCallback).await?;

        let (queue_name, _msg_count, _consumer_count) = match channel
            .queue_declare(QueueDeclareArguments::durable_client_named(queue))
            .await?
        {
            Some(a) => a,
            None => {
                return Err(anyhow!("queue declare returned None"));
            }
        };

        let routing_key = routing_key.to_string();
        let exchange_name = exchange_name.to_string();

        channel
            .exchange_declare(ExchangeDeclareArguments::new(&exchange_name, "direct"))
            .await?;

        channel
            .queue_bind(QueueBindArguments::new(
                &queue_name,
                &exchange_name,
                &routing_key,
            ))
            .await?;

        let consumer_tag = consumer_tag.to_string();

        Ok(RabbitMQ {
            channel,
            connection,
            queue: queue_name,
            routing_key,
            exchange_name,
            consumer_tag,
        })
    }

    pub async fn publish(&self, content: String) -> Result<()> {
        let args = BasicPublishArguments::new(&self.exchange_name, &self.routing_key);
        self.channel
            .basic_publish(BasicProperties::default(), content.into_bytes(), args)
            .await?;

        Ok(())
    }

    pub async fn basic_consume(
        &self,
        consumer_tag: &str,
        auto_ack: bool,
        sender: UnboundedSender<String>,
    ) -> Result<String> {
        let args = BasicConsumeArguments::new(&self.queue, consumer_tag)
            .auto_ack(auto_ack)
            .finish();

        let res = self
            .channel
            .basic_consume(Consumer::new(args.no_ack, sender), args)
            .await?;

        Ok(res)
    }

    pub async fn consume<C>(
        &self,
        consumer_tag: &str,
        auto_ack: bool,
        consumer: C,
    ) -> Result<String>
    where
        C: AsyncConsumer + Send + 'static,
    {
        let args = BasicConsumeArguments::new(&self.queue, consumer_tag)
            .auto_ack(auto_ack)
            .finish();

        let res = self.channel.basic_consume(consumer, args).await?;

        Ok(res)
    }
}

struct Consumer {
    no_ack: bool,
    sender: UnboundedSender<String>,
}

impl Consumer {
    pub fn new(no_ack: bool, sender: UnboundedSender<String>) -> Consumer {
        Consumer { no_ack, sender }
    }
}

#[async_trait]
impl AsyncConsumer for Consumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        // ack explicitly if manual ack
        if !self.no_ack {
            info!("ack to delivery {} on channel {}", deliver, channel);
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).await.unwrap();
        }

        let id = String::from_utf8(content).unwrap();
        if let Err(e) = self.sender.send(id) {
            error!("amq consumer: error when sending queue data through mpsc {e}");
        }
    }
}
