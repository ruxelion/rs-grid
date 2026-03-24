# Docker

The documentation site is packaged as a multi-stage Docker image. The build
stage compiles the Astro site with Node, and the serve stage runs nginx to
serve the static output.

## Quick start

From the repository root:

```bash
docker compose up --build
```

The site will be available at `http://localhost:8080`.

## docker-compose.yml

```yaml title="docker-compose.yml"
services:
  site:
    build:
      context: ./site
      dockerfile: Dockerfile
    ports:
      - "8080:80"
    restart: unless-stopped
```

## Dockerfile

Multi-stage build — Node builds, nginx serves:

```dockerfile title="site/Dockerfile"
# Stage 1: Build
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Stage 2: Serve
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
```

## nginx.conf

```nginx title="site/nginx.conf"
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ $uri.html =404;
    }

    # Long-cache for hashed Astro assets
    location /_astro/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    error_page 404 /404.html;
}
```

## Build without Compose

```bash
# Build the image
docker build -t rs-grid-site ./site

# Run the container
docker run -p 8080:80 rs-grid-site
```

## Production deployment

Push the image to a registry and deploy with your preferred orchestrator:

```bash
docker build -t registry.example.com/rs-grid-site:latest ./site
docker push registry.example.com/rs-grid-site:latest
```

:::tip
The final image is based on `nginx:alpine` and typically weighs around 25 MB.
No Node runtime is included in the production image.
:::
