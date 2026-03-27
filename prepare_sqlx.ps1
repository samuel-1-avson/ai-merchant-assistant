# SQLx Preparation Script
# Checks that the project compiles correctly with the database

Write-Host "=== SQLx Preparation ===" -ForegroundColor Green

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
    Write-Host "Please set it first: `$env:DATABASE_URL='your-connection-string'" -ForegroundColor Yellow
    exit 1
}

Write-Host "Using DATABASE_URL: $($env:DATABASE_URL.Substring(0, 50))..." -ForegroundColor Cyan

# Navigate to backend directory
cd backend

# Update dependencies
Write-Host "`nUpdating dependencies..." -ForegroundColor Yellow
cargo update

# Check compilation
Write-Host "`nChecking compilation..." -ForegroundColor Green
$checkResult = cargo check 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Project compiles successfully!" -ForegroundColor Green
    Write-Host "You can now run: docker-compose up --build" -ForegroundColor Green
} else {
    Write-Host "`n❌ Compilation failed:" -ForegroundColor Red
    Write-Host $checkResult -ForegroundColor Red
    
    Write-Host "`nMake sure:" -ForegroundColor Yellow
    Write-Host "  1. Your database is accessible from this machine" -ForegroundColor Yellow
    Write-Host "  2. DATABASE_URL is correct" -ForegroundColor Yellow
    Write-Host "  3. Database schema is set up (run supabase_schema.sql)" -ForegroundColor Yellow
}

cd ..
