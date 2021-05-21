FROM rust:1-slim-buster as Builder

WORKDIR /usr/src/webhook
USER root
COPY . .
RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release
FROM debian:buster-slim
WORKDIR /webhook
RUN apt-get update && apt-get install -y ca-certificates libreadline7 libssl1.1 pkg-config && rm -rf /var/lib/apt/lists/*  
COPY --from=Builder /usr/src/webhook/target/release/webhook .
COPY templates templates
EXPOSE 8080
CMD ["./webhook"]
