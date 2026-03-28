# Docker Deployment Guide - Updated for All 8 Fixes

This guide covers deploying the AI Merchant Assistant with all recent improvements including AI failover, fuzzy matching, transaction confirmations, alerts, OCR, multi-product transactions, and rate limiting.

## 🚀 What's New in This Update

### 1. AI Provider Failover System
- **Primary**: HuggingFace
- **Failover 1**: Together AI
- **Failover 2**: RunPod
- Circuit breaker pattern with auto-recovery

### 2. Fuzzy Product Matching
- Uses `sublime_fuzzy` for intelligent name matching
- Threshold: 60/100 for matches, 90/100 for auto-confirmation

### 3. Transaction Confirmation Flow
- 5-minute expiration for pending confirmations
- Auto-confirm at 90% confidence
- Manual approval for new products

### 4. Alert Detection Engine
- Low stock alerts
- Sales anomaly detection (Z-score based)
- Revenue comparison alerts
- Smart deduplication (24h window)

### 5. OCR Receipt Processing
- HuggingFace integration
- Structured receipt extraction
- Automatic product matching

### 6. Multi-Product Transactions
- Process multiple items in one voice command
- Example: "Sold 3 apples and 2 bananas"

### 7. Rate Limiting & Security
- Per-endpoint rate limits
- IP-based tracking
- Configurable windows and burst sizes

## 📋 Prerequisites

1. Docker and Docker Compose installed
2. HuggingFace API token (free)
3. Supabase project (database)
4. Optional: Together AI, RunPod, Google OAuth, GitHub OAuth tokens

## 🔧 Environment Setup

### Step 1: Update Your .env File

```bash
# Copy the example file
cp .env.example .env

# Edit with your actual values
nano .env
```

### Step 2: Required Variables

At minimum, you need these:

```env
# Database (Supabase)
DATABASE_URL=postgresql://...

# AI Provider (at least one)
HUGGINGFACE_API_TOKEN=hf_your_token

# Supabase
SUPABASE_URL=https://...
SUPABASE_SERVICE_KEY=...
SUPABASE_JWT_SECRET=...
SUPABASE_ANON_KEY=...

# Security
JWT_SECRET=your-super-secret-key-min-32-chars

# Frontend URLs
NEXT_PUBLIC_BACKEND_URL=http://localhost:8888
NEXT_PUBLIC_SUPABASE_URL=...
NEXT_PUBLIC_SUPABASE_ANON_KEY=...
```

### Step 3: Optional Variables (Recommended)

```env
# AI Failover (highly recommended)
TOGETHER_API_KEY=...
RUNPOD_ENDPOINT_URL=...
RUNPOD_API_KEY=...

# OAuth (for social login)
NEXT_PUBLIC_GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...

# Rate Limiting
RATE_LIMIT_AI_REQUESTS=20
RATE_LIMIT_AUTH_REQUESTS=10
RATE_LIMIT_DEFAULT_REQUESTS=100
```

## 🐳 Building and Running

### Development Mode

```bash
# Build and start all services
docker-compose up --build

# Run in detached mode
docker-compose up --build -d

# View logs
docker-compose logs -f backend
docker-compose logs -f frontend
```

### Production Mode

```bash
# Production deployment with all services
docker-compose -f docker-compose.prod.yml up --build -d

# View all logs
docker-compose -f docker-compose.prod.yml logs -f
```

### Individual Services

```bash
# Rebuild only backend
docker-compose up --build backend

# Rebuild only frontend
docker-compose up --build frontend

# Restart a service
docker-compose restart backend
```

## 🔍 Verifying the Deployment

### Health Checks

```bash
# Backend health
curl http://localhost:8888/health

# Frontend (should return HTML)
curl -I http://localhost:8889
```

### Test New Features

1. **AI Failover**:
   ```bash
   curl -X POST http://localhost:8888/api/v1/transactions/voice \
     -H "Content-Type: application/json" \
     -d '{"audio_data": "test", "user_id": "test"}' \
     -v
   ```

2. **Fuzzy Matching**:
   ```bash
   curl "http://localhost:8888/api/v1/products/search?q=aple"
   ```

3. **OCR Receipt**:
   ```bash
   curl -X POST http://localhost:8888/api/v1/ocr/receipt \
     -F "image=@receipt.jpg"
   ```

4. **Alerts**:
   ```bash
   curl http://localhost:8888/api/v1/alerts
   curl http://localhost:8888/api/v1/alerts/counts
   ```

## 🛠️ Troubleshooting

### Issue: Backend fails to build

**Solution**: Ensure SQLX_OFFLINE is set
```bash
# Check .env has SQLX_OFFLINE=true
# Then rebuild
docker-compose down
docker-compose up --build backend
```

### Issue: Database connection errors

**Solution**: Verify DATABASE_URL format
```bash
# Test connection
docker-compose exec backend psql $DATABASE_URL -c "SELECT 1"
```

### Issue: AI requests failing

**Solution**: Check AI provider tokens
```bash
# View backend logs
docker-compose logs backend | grep -i "ai\|huggingface\|failover"
```

### Issue: Rate limit errors

**Solution**: Adjust rate limits in .env
```env
RATE_LIMIT_AI_REQUESTS=50
RATE_LIMIT_AUTH_REQUESTS=20
RATE_LIMIT_DEFAULT_REQUESTS=200
```

### Issue: Frontend can't reach backend

**Solution**: Check environment variables
```bash
# Verify in .env
NEXT_PUBLIC_BACKEND_URL=http://localhost:8888

# Rebuild frontend
docker-compose up --build frontend
```

## 📊 Monitoring

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend

# Last 100 lines
docker-compose logs --tail=100 backend
```

### Resource Usage

```bash
# Docker stats
docker stats

# Container resource limits are set in docker-compose.prod.yml
```

## 🔄 Updating After Code Changes

```bash
# Pull latest code
git pull

# Rebuild and restart
docker-compose down
docker-compose up --build -d

# Or for production
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml up --build -d
```

## 🧹 Cleanup

```bash
# Stop containers
docker-compose down

# Stop and remove volumes (deletes all data!)
docker-compose down -v

# Remove all images
docker-compose down --rmi all

# Prune unused images
docker image prune -f
```

## 🌐 Accessing the Application

| Service | URL | Description |
|---------|-----|-------------|
| Frontend | http://localhost:8889 | Next.js web app |
| Backend API | http://localhost:8888/api/v1 | Rust API |
| Health Check | http://localhost:8888/health | Status endpoint |
| WebSocket | ws://localhost:8888/ws | Real-time updates |

## 📞 Support

If you encounter issues:
1. Check logs: `docker-compose logs -f`
2. Verify .env configuration
3. Ensure all required tokens are set
4. Check Docker resource limits

## 📝 Summary of Changes

| Fix | Feature | Docker Impact |
|-----|---------|---------------|
| #1 | AI Failover | New env vars: TOGETHER_API_KEY, RUNPOD_* |
| #2 | Fuzzy Matching | Built into backend, no config needed |
| #3 | Transaction Confirmations | Built-in, 5min expiration |
| #4 | Alert Engine | Built-in, auto-checks enabled |
| #5 | OCR Receipt | Built-in, HuggingFace integration |
| #6 | Multi-Product | Built-in, process multiple items |
| #7 | Frontend Button | Fixed in latest build |
| #8 | Rate Limiting | New env vars: RATE_LIMIT_* |
