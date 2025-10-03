# Use Rust official image as the base image for building
FROM rust:1.75 as builder

# Install required packages
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Create app directory
WORKDIR /app

# Copy files (except Cargo.lock to avoid version conflicts)
COPY Cargo.toml ./
COPY src/ ./src/
COPY index.html ./
COPY export_query.sql ./
COPY export_index.sql ./

# Build the WASM package (this will generate a new Cargo.lock)
RUN mkdir -p pkg && \
    wasm-pack build --target web --out-dir pkg
# Use nginx for serving the static files
FROM nginx:alpine

# Copy the built files to nginx
COPY --from=builder /app/index.html /usr/share/nginx/html/
COPY --from=builder /app/export_query.sql /usr/share/nginx/html/
COPY --from=builder /app/export_index.sql /usr/share/nginx/html/
COPY --from=builder /app/pkg /usr/share/nginx/html/pkg

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
