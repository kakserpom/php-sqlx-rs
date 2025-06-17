FROM php:8.4-cli AS build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      curl \
      clang \
      build-essential && \
    rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN mv /usr/local/etc/php/php.ini-production /usr/local/etc/php/php.ini
RUN cargo install cargo-php --locked
COPY ./ ./
RUN cd php-sqlx-cdylib && cargo php install --release --yes
