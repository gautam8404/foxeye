#!/usr/bin/env bash

FLAG=$0

source set_gcc.sh

export DATABASE_URL=postgresql://foxeye:foxeye@localhost:5432/foxeyedb
export RABBITMQ=amqp://foxeye:foxeye_amq@localhost:5672
export REDIS_URL=redis://localhost:6379/0
export RUST_LOG=info

cargo run --release
