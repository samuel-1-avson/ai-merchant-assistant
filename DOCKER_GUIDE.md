# AI Merchant Assistant - Docker Guide

This guide covers running the AI Merchant Assistant using Docker.

---

## Quick Start

### Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- `.env` file with proper configuration

### 1. Production Deployment

```bash
# Build and start all services
docker-compose up --build -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

**Access Points:**
- Frontend: http://localhost:3001
- Backend API: http://localhost:3000
- Health Check: http://localhost:3000/health

### 2. Development Mode (with Hot Reload)

```bash
# Start in development mode
docker-compose -f docker-compose.dev.yml up --build

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop
docker-compose -f docker-compose.dev.yml down
```

---

## Environment Variables

Create a `.env` file in the project root:

```env
# Database (Supabase PostgreSQL)
DATABASE_URL=postgresql://...

# Supabase
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SERVICE_KEY=your-service-key
SUPABASE_ANON_KEY=your-anon-key

# AI Provider
AI_PROVIDER=huggingface
HUGGINGFACE_API_TOKEN=hf_...

# JWT Secret (generate a strong random string)
JWT_SECRET=your-super-secret-jwt-key-min-32-chars

# Frontend
NEXT_PUBLIC_BACKEND_URL=http://localhost:3000
```

---

## Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Network                        │
│  ┌──────────────────┐      ┌──────────────────┐        │
│  │   Frontend       │      │    Backend       │        │
│  │   (Next.js)      │──────▶   (Rust/Axum)    │        │
│  │   Port: 3001     │      │   Port: 3000     │        │
│  └──────────────────┘      └────────┬─────────┘        │
│                                     │                   │
│                                     ▼                   │
│                           ┌──────────────────┐         │
│                           │   Supabase DB    │         │
│                           │   (PostgreSQL)   │         │
│                           └──────────────────┘         │
└─────────────────────────────────────────────────────────┘
```

---

## Commands Reference

### Build

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build backend
docker-compose build frontend

# Rebuild without cache
docker-compose build --no-cache
```

### Run

```bash
# Start in background
docker-compose up -d

# Start with build
docker-compose up --build -d

# Start specific service
docker-compose up -d backend
```

### Logs

```bash
# View all logs
docker-compose logs

# Follow logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f backend
docker-compose logs -f frontend

# View last 100 lines
docker-compose logs --tail=100
```

### Stop & Cleanup

```bash
# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Stop and remove everything including images
docker-compose down --rmi all -v
```

### Health Checks

```bash
# Check backend health
curl http://localhost:3000/health

# Check frontend
curl http://localhost:3001
```

---

## Troubleshooting

### Issue: Backend fails to start

**Check logs:**
```bash
docker-compose logs backend
```

**Common causes:**
- Database URL incorrect
- SQLX_OFFLINE not set
- Missing environment variables

**Solution:**
```bash
# Verify environment
docker-compose config

# Check if all env vars are set
cat .env
```

### Issue: Frontend can't connect to backend

**Check:**
```bash
# Verify backend is running
curl http://localhost:3000/health

# Check frontend logs
docker-compose logs frontend
```

**Solution:**
Ensure `NEXT_PUBLIC_API_URL` is set correctly in docker-compose.yml

### Issue: Port already in use

```bash
# Find process using port 3000
lsof -i :3000

# Kill process
kill -9 <PID>

# Or use different ports in docker-compose.yml
```

### Issue: Permission denied (Linux)

```bash
# Fix permissions
sudo chown -R $USER:$USER .

# Or run with sudo (not recommended for production)
sudo docker-compose up
```

---

## Performance Optimization

### 1. Multi-stage Builds

Both frontend and backend use multi-stage builds to minimize image size:

- **Frontend**: 3 stages (deps → builder → runner)
- **Backend**: 2 stages (builder → runtime)

### 2. Layer Caching

Dockerfile optimization for layer caching:
- Dependencies installed before source code
- Cargo.toml copied before src/ in backend

### 3. Health Checks

Services include health checks:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  timeout: 10s
  retries: 3
```

---

## Security Best Practices

1. **Non-root users**: Both services run as non-root
2. **Secrets**: Use `.env` file, never commit secrets
3. **Network**: Services isolated in Docker network
4. **Health checks**: Ensure services are healthy before use

---

## Updating Deployments

### Rolling Update (Zero Downtime)

```bash
# Pull latest changes
git pull

# Rebuild and restart
docker-compose up --build -d

# Verify health
curl http://localhost:3000/health
curl http://localhost:3001
```

### Database Migrations

```bash
# Connect to database
docker-compose exec backend psql $DATABASE_URL

# Or use Supabase dashboard for migrations
```

---

## Monitoring

### Resource Usage

```bash
# View container stats
docker stats

# View specific container
docker stats ai-merchant-backend
```

### Logs Aggregation

```bash
# Export logs
docker-compose logs > logs.txt

# View errors only
docker-compose logs | grep ERROR
```

---

## Production Checklist

- [ ] Strong JWT_SECRET set (min 32 chars)
- [ ] DATABASE_URL uses SSL
- [ ] HUGGINGFACE_API_TOKEN is valid
- [ ] CORS configured for production domain
- [ ] Health checks passing
- [ ] Resource limits set
- [ ] Monitoring configured
- [ ] Backups scheduled

---

## Support

For issues or questions:
1. Check logs: `docker-compose logs`
2. Verify environment: `docker-compose config`
3. Check health: `curl http://localhost:3000/health`

---

*Last Updated: March 27, 2026*
