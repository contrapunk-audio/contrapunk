# Multi-stage build: WASM -> SvelteKit frontend -> nginx

# Stage 1: Build WASM module with wasm-pack
FROM rust:1.88 AS wasm-builder
RUN cargo install wasm-pack
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY wasm/ wasm/
RUN wasm-pack build wasm/ --target web --out-dir /app/ui/src/lib/wasm-pkg --release

# Stage 2: Build SvelteKit frontend
FROM node:22 AS frontend-builder
WORKDIR /app/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci
COPY ui/ .
# Copy wasm-pack output into the SvelteKit source tree
COPY --from=wasm-builder /app/ui/src/lib/wasm-pkg src/lib/wasm-pkg/
# Build SvelteKit directly (skip prebuild since wasm-pkg is already provided)
RUN npx vite build

# Stage 3: Serve with nginx
FROM nginx:alpine
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=frontend-builder /app/ui/build /usr/share/nginx/html
EXPOSE 8080
