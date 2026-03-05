FROM --platform=linux/amd64 ubuntu:20.04

WORKDIR /app

RUN apt-get update
RUN apt-get -y install less python netcat

COPY ./amps_binary/AMPS-5.3.4.69-Release-Linux /app
COPY ./config/env.xml.xml /app/config/env.xml
RUN chmod +x /app/bin/ampServer

CMD ["/app/bin/ampServer", "/app/config/env.xml"]
