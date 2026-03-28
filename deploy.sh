#!/bin/bash
# AI Merchant Assistant - Docker Deployment Script
# Usage: ./deploy.sh [dev|prod|down|logs|clean]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if .env exists
check_env() {
    if [ ! -f .env ]; then
        log_error ".env file not found!"
        log_info "Please copy .env.example to .env and configure it:"
        log_info "  cp .env.example .env"
        exit 1
    fi
    
    # Check required variables
    if ! grep -q "HUGGINGFACE_API_TOKEN" .env || grep -q "HUGGINGFACE_API_TOKEN=$" .env; then
        log_warn "HUGGINGFACE_API_TOKEN not set in .env"
        log_info "Get your token from: https://huggingface.co/settings/tokens"
    fi
    
    if ! grep -q "DATABASE_URL" .env || grep -q "DATABASE_URL=$" .env; then
        log_error "DATABASE_URL not set in .env"
        exit 1
    fi
}

# Development mode
deploy_dev() {
    log_info "Starting development deployment..."
    check_env
    
    log_info "Building and starting services..."
    docker-compose down
    docker-compose up --build -d
    
    log_info "Waiting for services to be healthy..."
    sleep 5
    
    # Health check
    if curl -s http://localhost:8888/health > /dev/null; then
        log_success "Backend is running at http://localhost:8888"
    else
        log_warn "Backend may still be starting, check logs with: docker-compose logs -f backend"
    fi
    
    log_success "Frontend is running at http://localhost:8889"
    log_info "View logs: docker-compose logs -f"
}

# Production mode
deploy_prod() {
    log_info "Starting production deployment..."
    check_env
    
    log_info "Building and starting production services..."
    docker-compose -f docker-compose.prod.yml down
    docker-compose -f docker-compose.prod.yml up --build -d
    
    log_info "Waiting for services to be healthy..."
    sleep 10
    
    if curl -s http://localhost:3000/health > /dev/null; then
        log_success "Production backend is running at http://localhost:3000"
    else
        log_warn "Services may still be starting..."
    fi
    
    log_success "Production frontend is running at http://localhost:3001"
}

# Stop all services
stop_services() {
    log_info "Stopping all services..."
    docker-compose down
    docker-compose -f docker-compose.prod.yml down 2>/dev/null || true
    log_success "All services stopped"
}

# View logs
view_logs() {
    if [ -z "$2" ]; then
        docker-compose logs -f
    else
        docker-compose logs -f "$2"
    fi
}

# Clean up
clean_up() {
    log_warn "This will remove all containers, volumes, and images!"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        docker-compose down -v --rmi all
        docker-compose -f docker-compose.prod.yml down -v --rmi all 2>/dev/null || true
        docker system prune -f
        log_success "Cleanup complete"
    else
        log_info "Cleanup cancelled"
    fi
}

# Main command handler
case "${1:-dev}" in
    dev)
        deploy_dev
        ;;
    prod)
        deploy_prod
        ;;
    down|stop)
        stop_services
        ;;
    logs)
        view_logs "$@"
        ;;
    clean)
        clean_up
        ;;
    restart)
        stop_services
        deploy_dev
        ;;
    status)
        docker-compose ps
        ;;
    build)
        docker-compose build --no-cache
        ;;
    *)
        echo "Usage: $0 [dev|prod|down|logs|clean|restart|status|build]"
        echo ""
        echo "Commands:"
        echo "  dev      - Deploy in development mode (default)"
        echo "  prod     - Deploy in production mode"
        echo "  down     - Stop all services"
        echo "  logs     - View logs (optionally: logs backend)"
        echo "  clean    - Remove all containers, volumes, and images"
        echo "  restart  - Restart all services"
        echo "  status   - Show service status"
        echo "  build    - Rebuild images without cache"
        exit 1
        ;;
esac
