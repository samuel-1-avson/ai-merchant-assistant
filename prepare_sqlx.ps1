# SQLx Offline Preparation Script
# This generates the .sqlx folder needed for Docker builds

Write-Host "=== SQLx Offline Preparation ===" -ForegroundColor Green

# Check if DATABASE_URL is set
if (-not $env:DATABASE_URL) {
    Write-Host "DATABASE_URL not set. Loading from .env file..." -ForegroundColor Yellow
    
    # Try to load from .env file
    if (Test-Path .env) {
        $envContent = Get-Content .env
        foreach ($line in $envContent) {
            if ($line -match '^DATABASE_URL=(.+)$') {
                $env:DATABASE_URL = $matches[1]
                Write-Host "Loaded DATABASE_URL from .env" -ForegroundColor Green
                break
            }
        }
    }
}

if (-not $env:DATABASE_URL) {
    Write-Host "ERROR: DATABASE_URL environment variable is not set!" -ForegroundColor Red
    Write-Host "Please set it first: $env:DATABASE_URL='your-connection-string'" -ForegroundColor Yellow
    exit 1
}

Write-Host "Using DATABASE_URL: $($env:DATABASE_URL.Substring(0, 50))..." -ForegroundColor Cyan

# Navigate to backend directory
cd backend

# Install sqlx-cli if not already installed
Write-Host "`nChecking sqlx-cli..." -ForegroundColor Yellow
$cargoInstall = cargo install --list | Select-String "sqlx-cli"
if (-not $cargoInstall) {
    Write-Host "Installing sqlx-cli..." -ForegroundColor Yellow
    cargo install sqlx-cli
}

# Run cargo sqlx prepare
Write-Host "`nRunning cargo sqlx prepare..." -ForegroundColor Green
$sqlxResult = cargo sqlx prepare 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ SQLx offline data generated successfully!" -ForegroundColor Green
    Write-Host "The .sqlx folder has been created and will be used in Docker builds." -ForegroundColor Green
} else {
    Write-Host "`n❌ Failed to generate SQLx offline data:" -ForegroundColor Red
    Write-Host $sqlxResult -ForegroundColor Red
    
    Write-Host "`nMake sure:" -ForegroundColor Yellow
    Write-Host "  1. Your database is accessible from this machine" -ForegroundColor Yellow
    Write-Host "  2. DATABASE_URL is correct" -ForegroundColor Yellow
    Write-Host "  3. Database schema is set up (run supabase_schema.sql)" -ForegroundColor Yellow
}

cd ..
