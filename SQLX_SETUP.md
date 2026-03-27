# SQLx Setup Guide

## Overview

This project uses SQLx for compile-time checked SQL queries. This means the database must be accessible during compilation.

## How It Works

The Docker build now passes `DATABASE_URL` as a build argument, so SQLx can connect to your Supabase database during the build process.

## Prerequisites

1. Database schema created in Supabase (run `supabase_schema.sql`)
2. `DATABASE_URL` set in `.env` file
3. Your machine can access the internet (to reach Supabase)

## Build Instructions

### Option 1: Build with Docker (Recommended)

```bash
docker-compose up --build
```

The `docker-compose.yml` passes the `DATABASE_URL` to the build process automatically.

### Option 2: Test Compilation Locally

```powershell
# Set environment variable
$env:DATABASE_URL="postgresql://postgres.mufjqnudxzkaandzohbj:Deanhall360@@@aws-1-eu-west-1.pooler.supabase.com:6543/postgres"

# Run the preparation script
.\prepare_sqlx.ps1
```

Or manually:

```powershell
cd backend
cargo update
cargo check
cd ..
```

---

## Troubleshooting

### "database does not exist" Error
Make sure you've run the schema SQL in Supabase SQL Editor first.

### Connection Timeout
- Supabase connection pooler may take time to warm up
- Check your internet connection
- Verify the database password is correct

### "failed to select a version for sqlx"
Run `cargo update` to refresh dependencies:
```bash
cd backend
cargo update
```

---

## Architecture

```
Docker Build
    │
    ├─► Pass DATABASE_URL as build arg
    │
    ├─► Rust compiles with SQLx
    │   └─► SQLx connects to Supabase
    │   └─► Validates queries at compile time
    │
    └─► Creates optimized binary
```

---

## Security Notes

- The `DATABASE_URL` is only used during build, not included in final image
- Service credentials are passed via environment variables at runtime
- Never commit `.env` file to Git
