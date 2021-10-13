FROM rust:slim AS Builder

WORKDIR /obelisk

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN mkdir src
# We need a proper main.rs to make the first cargo build run
RUN echo 'fn main() {println!("Hello World!");}' > src/main.rs
# As we haven't copied any source files yet, this will only compile the dependencies
RUN cargo build --release

# Now copy the real source over
COPY src src
# Cargo detects changes only by mtime
RUN touch src/main.rs
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libpq5

WORKDIR /obelisk

COPY --from=Builder /obelisk/target/release/k3k-obelisk .

# Unicode Dagger/Obelisk = U+2020
EXPOSE 2020
ENTRYPOINT [ "./k3k-obelisk" ]
