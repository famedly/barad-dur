FROM docker.io/rustlang/rust:nightly-slim
COPY ./target/release/barad-dur /usr/local/bin/barad-dur
CMD ["/usr/local/bin/barad-dur"]