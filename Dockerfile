# Stage 1: Build environment
FROM clarkenciel/rust-builder:rust-1.82 AS rust-base

FROM rust-base AS builder

# Install Node.js and pnpm for frontend dependencies
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g pnpm

# Install trunk from pre-built binary (faster and uses less memory)
RUN curl -L https://github.com/trunk-rs/trunk/releases/download/v0.21.3/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C /usr/local/bin

# Set working directory
WORKDIR /app

# Copy package files and install Node dependencies first (for better caching)
COPY package.json pnpm-lock.yaml* ./
RUN pnpm install

# Copy Rust project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY index.html ./
COPY Trunk.toml ./
COPY input.css ./
COPY assets ./assets

# Build the application in release mode
RUN trunk build --release

# Stage 2: Production server with NGINX
FROM nginx:alpine

# Copy the built assets from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]