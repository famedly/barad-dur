FROM docker.io/alpine:edge as builder
RUN apk add --no-cache \
      cargo \
      build-base \
      openssl-dev \
      cmake \
      git
COPY . /app
WORKDIR /app
RUN cargo build --release

FROM docker.io/alpine:edge
RUN apk add --no-cache \
      libgcc \
      libssl1.1 \
      curl \
  && mkdir -p /opt/barad-dur
WORKDIR /opt/barad-dur
COPY --from=builder /app/target/release/barad-dur /usr/local/bin/barad-dur
CMD ["/usr/local/bin/barad-dur"]
