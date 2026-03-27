# Docker Setup Guide

## Quick Start

### 1. Set Up Environment Variables

```bash
# Copy the example env file
cp backend/.env.example backend/.env

# Or use the root .env file (already created for local dev)
```

### 2. Get a HuggingFace API Token (FREE)

1. Go to https://huggingface.co/settings/tokens
2. Create a new token (read-only is fine)
3. Add it to your `.env` file:
   ```
   HUGGINGFACE_API_TOKEN=hf_your_actual_token_here
   ```

### 3. Build and Run

```bash
# Build and start all services
docker-compose up --build

# Or run in detached mode
docker-compose up --build -d
```

### 4. Access the Application

- **Frontend:** http://localhost:3001
- **Backend API:** http://localhost:3000
- **API Health Check:** http://localhost:3000/health

---

## Environment Variables Reference

| Variable | Required | Description |
|----------|----------|-------------|
| `HUGGINGFACE_API_TOKEN` | Yes | Get free token from huggingface.co |
| `AI_PROVIDER` | No | Default: `huggingface` |
| `SUPABASE_URL` | No | Optional for local dev |
| `SUPABASE_ANON_KEY` | No | Optional for local dev |

---

## Troubleshooting

### Issue: `npm ci` fails
**Solution:** Fixed in Dockerfile - now uses `npm install`

### Issue: Missing package-lock.json
**Solution:** Dockerfile now uses `npm install` instead of `npm ci`

### Issue: Backend fails to connect to database
**Solution:** Wait for PostgreSQL to be ready (it takes ~10 seconds on first run)

### Issue: "variable is not set" warnings
**Solution:** These are just warnings. For local development, the defaults work fine.

---

## Stopping the Application

```bash
# Stop containers
docker-compose down

# Stop and remove volumes (deletes database data)
docker-compose down -v
```

## Rebuilding After Code Changes

```bash
# Rebuild specific service
docker-compose up --build backend

# Rebuild everything
docker-compose up --build
```
