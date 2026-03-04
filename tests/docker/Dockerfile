FROM --platform=linux/amd64 ubuntu:20.04

WORKDIR /app

RUN apt-get update
RUN apt-get -y install less python netcat

COPY ./downloaded_folders/AMPS-5.3.4.69-Release-Linux /app
COPY ./amps-config.xml /app/config/amps-config.xml
RUN chmod +x /app/bin/ampServer

CMD ["/app/bin/ampServer", "/app/config/amps-config.xml"]
