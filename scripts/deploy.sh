#!/bin/bash
set -e

# Production Deployment Script
# Usage: ./deploy.sh [environment]
# Environment: production (default), staging

ENVIRONMENT=${1:-production}
VERSION=${2:-latest}

echo "🚀 Deploying AI Merchant Assistant to $ENVIRONMENT"
echo "Version: $VERSION"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(production|staging)$ ]]; then
    echo "❌ Invalid environment. Use 'production' or 'staging'"
    exit 1
fi

# Load environment variables
if [ -f ".env.$ENVIRONMENT" ]; then
    echo "📋 Loading environment variables..."
    export $(cat .env.$ENVIRONMENT | xargs)
else
    echo "❌ Environment file .env.$ENVIRONMENT not found"
    exit 1
fi

# Build and tag images
echo "🔨 Building Docker images..."
docker build -t aimerchant/backend:$VERSION -f Dockerfile.backend .
docker build -t aimerchant/frontend:$VERSION -f Dockerfile.frontend .

# Push to registry
echo "📤 Pushing images to registry..."
docker push aimerchant/backend:$VERSION
docker push aimerchant/frontend:$VERSION

# Deploy to Kubernetes
echo "☸️  Deploying to Kubernetes..."
kubectl apply -f k8s/namespace.yml
kubectl apply -f k8s/configmap.yml
kubectl apply -f k8s/secret.yml

# Update deployments with new version
kubectl set image deployment/backend backend=aimerchant/backend:$VERSION -n ai-merchant
kubectl set image deployment/frontend frontend=aimerchant/frontend:$VERSION -n ai-merchant

# Wait for rollout
echo "⏳ Waiting for rollout to complete..."
kubectl rollout status deployment/backend -n ai-merchant --timeout=300s
kubectl rollout status deployment/frontend -n ai-merchant --timeout=300s

# Verify deployment
echo "✅ Verifying deployment..."
kubectl get pods -n ai-merchant
kubectl get svc -n ai-merchant

# Health check
echo "🏥 Running health checks..."
sleep 10
if curl -sf http://localhost:3000/health > /dev/null; then
    echo "✅ Backend is healthy"
else
    echo "❌ Backend health check failed"
    exit 1
fi

echo "🎉 Deployment complete!"
