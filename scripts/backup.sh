#!/bin/bash
set -e

# Database Backup Script
# Usage: ./backup.sh [retention_days]

RETENTION_DAYS=${1:-30}
BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="ai_merchant_backup_$TIMESTAMP.sql.gz"

echo "💾 Starting database backup..."
echo "Retention: $RETENTION_DAYS days"

# Create backup directory if not exists
mkdir -p $BACKUP_DIR

# Perform backup
echo "📦 Creating backup..."
pg_dump $DATABASE_URL | gzip > $BACKUP_DIR/$BACKUP_FILE

# Upload to S3
echo "☁️  Uploading to S3..."
aws s3 cp $BACKUP_DIR/$BACKUP_FILE s3://$BACKUP_BUCKET/backups/

# Clean up old backups locally
echo "🧹 Cleaning up old local backups..."
find $BACKUP_DIR -name "ai_merchant_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Clean up old backups in S3
echo "🧹 Cleaning up old S3 backups..."
aws s3 ls s3://$BACKUP_BUCKET/backups/ | awk '{print $4}' | while read file; do
    file_date=$(echo $file | grep -o '[0-9]\{8\}')
    file_timestamp=$(date -d $file_date +%s)
    current_timestamp=$(date +%s)
    age_days=$(( (current_timestamp - file_timestamp) / 86400 ))
    
    if [ $age_days -gt $RETENTION_DAYS ]; then
        echo "Deleting old backup: $file"
        aws s3 rm s3://$BACKUP_BUCKET/backups/$file
    fi
done

echo "✅ Backup complete: $BACKUP_FILE"
