FROM rust:1-slim-buster

RUN apt-get update && apt-get upgrade && apt-get install git -y

RUN rustup install nightly && rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force

ADD . mynachain
WORKDIR mynachain
RUN cargo build --release

EXPOSE 9944

CMD ["bash"]
