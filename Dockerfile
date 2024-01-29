FROM docker.io/alpine:3.14 as builder

RUN apk add --no-cache \
	rustup \
	build-base \
	openssh-client-default \
	git
RUN rustup-init -y -q
RUN source $HOME/.cargo/env && cargo install cargo-auditable

COPY . /app
WORKDIR /app
RUN source $HOME/.cargo/env && cargo auditable build --release

FROM docker.io/alpine:3.14
RUN apk add --no-cache \
	libgcc \
	tzdata \
#IMPORTANT: in order for the Docker container to be able to perform the check, the image must provide `curl`.
#           If changing or updating the base image's version, please ensure that `curl` is available!
	curl && \
# ensure the UTC timezone is set
	ln -fs /usr/share/zoneinfo/Etc/UTC /etc/localtime

WORKDIR /opt/barad-dur
COPY --from=builder /app/target/release/barad-dur /usr/local/bin/barad-dur
COPY --from=builder /app/migrations /opt/barad-dur/migrations
CMD ["/usr/local/bin/barad-dur"]

ENV TZ=Etc/UTC
ARG service_port_number=8080
EXPOSE ${service_port_number}/tcp
ENV SERVICE_PORT=${service_port_number}
HEALTHCHECK --interval=3s --timeout=3s --retries=2 --start-period=5s \
 CMD curl -fSs http://localhost:$SERVICE_PORT/health || exit 1
 