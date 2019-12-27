FROM debian:buster-slim

RUN apt-get update && apt-get upgrade && apt-get install git curl build-essential -y

RUN curl https://sh.rustup.rs -sSf | sh && rustup default stable
RUN ENV PATH $PATH:/root/.cargo/bin
RUN rustup install nightly && rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force

ADD . mynachain
WORKDIR mynachain
RUN cargo build --release

EXPOSE 9944

CMD ["bash"]
