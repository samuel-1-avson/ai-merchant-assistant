# SQLx Offline Mode Setup

## The Problem

SQLx performs compile-time query checking against a live database. In Docker builds, the database isn't accessible during compilation, so we need to use **offline mode**.

## Solution: Generate Offline Query Data

### Prerequisites

1. Rust toolchain installed locally
2. Database accessible (Supabase connection)
3. Database schema already created in Supabase

### Step 1: Install sqlx-cli

```powershell
cargo install sqlx-cli
```

### Step 2: Set Environment Variable

```powershell
$env:DATABASE_URL="postgresql://postgres.mufjqnudxzkaandzohbj:Deanhall360@@@aws-1-eu-west-1.pooler.supabase.com:6543/postgres"
```

### Step 3: Generate Offline Data

```powershell
cd backend
cargo sqlx prepare
cd ..
```

This creates a `.sqlx/` folder with query metadata.

### Step 4: Build Docker

```bash
docker-compose up --build
```

---

## Alternative: Quick Fix (Skip Compile-Time Checks)

If you want to skip SQLx compile-time checks entirely:

### Option A: Use Runtime Queries (Modify Code)

Change `sqlx::query!` to `sqlx::query` and `sqlx::query_as!` to `sqlx::query_as` in the source files. This removes compile-time checking.

### Option B: Use Build Args with Database URL

The Dockerfile already supports passing DATABASE_URL:

```bash
docker-compose build --build-arg DATABASE_URL=$env:DATABASE_URL
docker-compose up
```

---

## Troubleshooting

### "database does not exist" Error
Make sure you've run the schema SQL in Supabase first.

### "permission denied" Error
Check that your Supabase service_role key has the necessary permissions.

### Connection Timeout
The Supabase connection pooler may take time to warm up. Try again.

---

## Automated Script

Run the provided PowerShell script:

```powershell
.\prepare_sqlx.ps1
```

This will:
1. Load DATABASE_URL from .env
2. Install sqlx-cli if needed
3. Run `cargo sqlx prepare`
4. Generate the .sqlx folder
