#!/bin/bash
set -e

# Database Migration Script
# Usage: ./migrate.sh [command]
# Commands: up, down, status, create [name]

COMMAND=${1:-status}
MIGRATION_DIR="backend/src/db/migrations"

case $COMMAND in
    up)
        echo "⬆️  Running migrations..."
        sqlx migrate run --source $MIGRATION_DIR
        ;;
    down)
        echo "⬇️  Reverting last migration..."
        sqlx migrate revert --source $MIGRATION_DIR
        ;;
    status)
        echo "📋 Migration status..."
        sqlx migrate info --source $MIGRATION_DIR
        ;;
    create)
        NAME=$2
        if [ -z "$NAME" ]; then
            echo "❌ Migration name required"
            exit 1
        fi
        echo "📝 Creating migration: $NAME"
        sqlx migrate add --source $MIGRATION_DIR $NAME
        ;;
    *)
        echo "Usage: $0 {up|down|status|create [name]}"
        exit 1
        ;;
esac

echo "✅ Done"
