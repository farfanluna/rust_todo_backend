FROM rustlang/rust:nightly-bookworm

WORKDIR /app

# Install sqlx-cli
RUN cargo install sqlx-cli

# Copy source code
COPY ./backend /app/backend

WORKDIR /app/backend

# Build the application
RUN cargo build --release

EXPOSE 3000

CMD ["/app/backend/target/release/rust_todo_backend"]
