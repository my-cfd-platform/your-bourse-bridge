FROM ubuntu:22.04
COPY ./target/release/your-bourse-bridge ./target/release/your-bourse-bridge

ENTRYPOINT ["./target/release/your-bourse-bridge"]