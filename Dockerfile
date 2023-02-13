FROM rust:slim
COPY ./target/release/your-borse-bridge ./target/release/your-borse-bridge

ARG app_version
ARG app_compilation_date
ENV APP_VERSION=${app_version}
ENV APP_COMPILATION_DATE=${app_compilation_date}

ENTRYPOINT ["./target/release/your-borse-bridge"]