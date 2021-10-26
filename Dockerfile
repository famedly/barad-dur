FROM docker.io/alpine:3.14 as builder

RUN apk add --no-cache \
	rustup \
	build-base \
	openssh-client-default \
	git
RUN rustup-init -y -q

COPY . /app
WORKDIR /app
RUN source $HOME/.cargo/env && cargo build --release

FROM docker.io/alpine:3.14
RUN apk add --no-cache \
	libgcc
WORKDIR /opt/barad-dur
COPY --from=builder /app/target/release/barad-dur /usr/local/bin/barad-dur
COPY --from=builder /app/migrations /opt/barad-dur/migrations
CMD ["/usr/local/bin/barad-dur"]
