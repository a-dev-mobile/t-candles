# Сборка приложения
# https://hub.docker.com/_/rust/tags
FROM rust:1.85.0 AS builder

WORKDIR /app

# Добавляем переменную окружения для сборки
ARG RUST_ENV=production
ENV RUST_ENV=${RUST_ENV}

# 1. Копируем только файлы для зависимостей
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# 2. Создаем фиктивную структуру src для сборки зависимостей
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 3. Теперь копируем реальный исходный код
COPY src ./src
COPY config ./config
COPY .sqlx .sqlx/

# 4. Выполняем финальную сборку
RUN cargo build --release

# Stage 2: Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/t-candles .
COPY --from=builder /app/config /usr/local/bin/config

EXPOSE 5000

ENV RUST_ENV=production

CMD ["./t-candles"]