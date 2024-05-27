use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{BasicConsumeArguments, BasicPublishArguments, Channel, ExchangeDeclareArguments, QueueBindArguments, QueueDeclareArguments};
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::consumer::DefaultConsumer;
use amqprs::BasicProperties;
use anyhow::{anyhow, Result};
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct RabbitMQ {
    channel: Channel,
    connection: Connection,
    queue: String,
    routing_key: String,
    exchange_name: String,
    consumer_tag: String,
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
    pub async fn new(uri: &str, queue: &str, consumer_tag: &str) -> Result<RabbitMQ> {
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

        let routing_key = "foxeye.routing".to_string();
        let exchange_name = "foxeye.topic".to_string();
        
        channel.exchange_declare(ExchangeDeclareArguments::new(&exchange_name, "direct")).await?;


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

    pub async fn consume(&self) -> Result<String> {
        let args = BasicConsumeArguments::new(&self.queue, &self.consumer_tag);
        let res = self
            .channel
            .basic_consume(DefaultConsumer::new(args.no_ack), args)
            .await?;

        Ok(res)
    }
}
