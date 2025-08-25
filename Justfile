build-linux:
    cross build --target x86_64-unknown-linux-gnu --release

# Build Docker image
docker-build:
    docker build -t ghcr.io/divvun/divvun-worker-static:latest .

# Push Docker image to GitHub Container Registry
docker-push:
    docker push ghcr.io/divvun/divvun-worker-static:latest

# Build and push Docker image
docker-build-push: docker-build docker-push
