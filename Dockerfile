FROM php:8.4-cli AS build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      curl \
      git \
      clang \
      build-essential && \
    rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN mv /usr/local/etc/php/php.ini-production /usr/local/etc/php/php.ini
RUN cargo install cargo-php --locked
WORKDIR /opt/php-sqlx
COPY . .
RUN cd php-sqlx-cdylib && cargo php install --release --yes
WORKDIR /opt/php-sqlx/benches
RUN curl -s https://raw.githubusercontent.com/composer/getcomposer.org/f3108f64b4e1c1ce6eb462b159956461592b3e3e/web/installer | php -- --quiet
RUN ./composer.phar require phpbench/phpbench --dev
CMD ["./vendor/bin/phpbench run benchmark.php"]
