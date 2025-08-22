FROM rustlang/rust:nightly-bookworm

WORKDIR /app

# Install sqlx-cli
RUN cargo install sqlx-cli

# Copy Cargo files first for better caching
COPY ./backend/Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm -rf src

# The actual source code will be mounted as a volume
# This allows for hot reloading during development

EXPOSE 3000

# Default command - will be overridden by docker-compose
CMD ["cargo", "run"]
