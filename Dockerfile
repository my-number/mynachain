FROM debian:buster-slim

ADD ./node-template ./node-template

RUN chmod +x ./node-template

EXPOSE 9944

CMD ["./node-template", "--dev", "--ws-external", "--rpc-external"]
