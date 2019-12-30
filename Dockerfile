FROM debian:buster-slim

RUN apt-get update && apt-get upgrade && apt install -y cmake pkg-config libssl-dev build-essential git clang libclang-dev

ADD ./node-template ./node-template

RUN chmod -x ./node-template

EXPOSE 9944

CMD ["./node-template"]
