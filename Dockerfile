FROM rust:1.92-alpine AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release --locked && rm -rf src
COPY src ./src
COPY public-html ./public-html
RUN touch src/main.rs && cargo build --release --locked

FROM gcr.io/distroless/static-debian13:nonroot
COPY --from=build /src/target/release/wednesday /wednesday
EXPOSE 8080
ENTRYPOINT ["/wednesday"]
