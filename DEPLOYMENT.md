# Production Deployment Guide

## Overview

This guide covers deploying AI Merchant Assistant to production using Docker Compose or Kubernetes.

## Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- kubectl 1.25+ (for Kubernetes)
- Helm 3.0+ (optional, for Kubernetes)
- Domain name with DNS access
- SSL certificates (Let's Encrypt or custom)

## Quick Start (Docker Compose)

### 1. Clone and Setup

```bash
git clone https://github.com/your-org/ai-merchant-assistant.git
cd ai-merchant-assistant

# Create environment file
cp .env.example .env.production

# Edit environment variables
nano .env.production
```

### 2. Configure Environment

```bash
# Required variables
DATABASE_URL=postgresql://user:password@db:5432/ai_merchant
REDIS_URL=redis://redis:6379
JWT_SECRET=your-super-secret-jwt-key
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SERVICE_KEY=your-service-key
AI_PROVIDER=huggingface
HUGGINGFACE_API_TOKEN=your-token
```

### 3. Deploy

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Deploy
./scripts/deploy.sh production
```

### 4. Verify

```bash
# Check services
docker-compose -f docker-compose.prod.yml ps

# Check logs
docker-compose -f docker-compose.prod.yml logs -f backend

# Health check
curl https://your-domain.com/health
```

## Kubernetes Deployment

### 1. Setup Cluster

```bash
# Create namespace
kubectl apply -f k8s/namespace.yml

# Create configmaps and secrets
kubectl apply -f k8s/configmap.yml
kubectl apply -f k8s/secret.yml
```

### 2. Deploy Applications

```bash
# Deploy backend and frontend
kubectl apply -f k8s/deployment-backend.yml
kubectl apply -f k8s/deployment-frontend.yml

# Deploy HPA
kubectl apply -f k8s/hpa.yml
```

### 3. Verify Deployment

```bash
# Check pods
kubectl get pods -n ai-merchant

# Check services
kubectl get svc -n ai-merchant

# Check HPA
kubectl get hpa -n ai-merchant
```

## SSL/TLS Configuration

### Let's Encrypt (Recommended)

```bash
# Initial certificate generation
docker-compose -f docker-compose.prod.yml run --rm certbot \
  certonly --webroot \
  --webroot-path=/var/www/certbot \
  --email admin@your-domain.com \
  --agree-tos \
  --no-eff-email \
  -d your-domain.com \
  -d www.your-domain.com
```

### Custom Certificates

```bash
# Place certificates in nginx/ssl/
cp your-cert.pem nginx/ssl/cert.pem
cp your-key.pem nginx/ssl/key.pem
```

## Monitoring Setup

### Prometheus & Grafana

```bash
# Deploy monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# Access Grafana
echo "http://localhost:3001"
# Default credentials: admin/admin
```

### Health Checks

```bash
# Backend health
curl https://api.your-domain.com/health

# Frontend health
curl https://your-domain.com/

# Database health
kubectl exec -it deploy/backend -n ai-merchant -- pg_isready
```

## Backup and Recovery

### Automated Backups

```bash
# Setup cron job for daily backups
0 2 * * * /path/to/ai-merchant-assistant/scripts/backup.sh >> /var/log/ai-merchant-backup.log 2>&1
```

### Manual Backup

```bash
./scripts/backup.sh
```

### Restore from Backup

```bash
# Download from S3
aws s3 cp s3://your-bucket/backups/ai_merchant_backup_20240115.sql.gz .

# Restore
gunzip < ai_merchant_backup_20240115.sql.gz | psql $DATABASE_URL
```

## Database Migrations

### Run Migrations

```bash
# Using script
./scripts/migrate.sh up

# Using sqlx directly
cd backend
sqlx migrate run
```

### Create New Migration

```bash
./scripts/migrate.sh create add_user_preferences
```

## Scaling

### Horizontal Scaling (Kubernetes)

```bash
# Manual scaling
kubectl scale deployment backend --replicas=5 -n ai-merchant

# View HPA status
kubectl get hpa -n ai-merchant
```

### Vertical Scaling

Edit deployment resources:

```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

## Troubleshooting

### Common Issues

#### 1. Database Connection Failed

```bash
# Check database is running
docker-compose -f docker-compose.prod.yml ps db

# Check logs
docker-compose -f docker-compose.prod.yml logs db

# Test connection
docker-compose -f docker-compose.prod.yml exec db pg_isready
```

#### 2. High Memory Usage

```bash
# Check memory usage
docker stats

# Restart service
docker-compose -f docker-compose.prod.yml restart backend
```

#### 3. SSL Certificate Issues

```bash
# Renew certificates
docker-compose -f docker-compose.prod.yml run --rm certbot renew

# Test SSL configuration
openssl s_client -connect your-domain.com:443 -servername your-domain.com
```

## Security Checklist

- [ ] Change default passwords
- [ ] Enable firewall (allow only 80, 443)
- [ ] Configure fail2ban
- [ ] Enable automatic security updates
- [ ] Setup log monitoring
- [ ] Configure backup encryption
- [ ] Review CORS settings
- [ ] Enable HSTS
- [ ] Setup DDoS protection (CloudFlare recommended)

## Maintenance Windows

### Regular Tasks

| Task | Frequency | Command |
|------|-----------|---------|
| Security updates | Weekly | `apt update && apt upgrade` |
| Log rotation | Daily | `logrotate -f /etc/logrotate.conf` |
| Database backup | Daily | `./scripts/backup.sh` |
| Certificate renewal | Weekly | `certbot renew` |
| Health check | Continuous | Built-in probes |

## Rollback Procedure

```bash
# Rollback to previous version
kubectl rollout undo deployment/backend -n ai-merchant

# Or specific revision
kubectl rollout history deployment/backend -n ai-merchant
kubectl rollout undo deployment/backend -n ai-merchant --to-revision=2
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/ai-merchant-assistant/issues
- Documentation: https://docs.aimerchant.app
- Email: support@aimerchant.app
