FROM cosmosio/rust:nightly
MAINTAINER Graham Lee <ghmlee@cosmos.io>

ADD . /curiosity
RUN cargo build --release --manifest-path /curiosity/Cargo.toml
WORKDIR /curiosity
CMD ["./target/release/curiosity"]