FROM ubuntu:20.04
WORKDIR /app
COPY target/release/with-baby-store .
ENTRYPOINT ["/app/with-baby-store"]        