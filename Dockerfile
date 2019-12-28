FROM debian:buster-slim

RUN apt-get update && apt-get upgrade && apt-get install git -y

ADD ./node-template ./node-template

RUN chmod -x ./node-template

EXPOSE 9944

CMD ["./node-template"]
