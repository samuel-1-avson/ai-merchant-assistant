# Operations Runbook

## Daily Operations

### Health Checks

```bash
# Check all services
docker-compose -f docker-compose.prod.yml ps

# Check backend health
curl -f https://api.aimerchant.app/health || echo "Backend unhealthy"

# Check database
psql $DATABASE_URL -c "SELECT COUNT(*) FROM users;"

# Check Redis
redis-cli -u $REDIS_URL ping
```

### Log Monitoring

```bash
# View recent logs
docker-compose -f docker-compose.prod.yml logs --tail=100

# Follow logs
docker-compose -f docker-compose.prod.yml logs -f

# Search for errors
docker-compose -f docker-compose.prod.yml logs | grep ERROR
```

### Metrics Review

Check these metrics daily:
- Request rate and error rate
- Response time (p50, p95, p99)
- CPU and memory usage
- Database connections
- Disk space

## Incident Response

### Severity Levels

| Level | Description | Response Time |
|-------|-------------|---------------|
| P0 | Complete outage | 15 minutes |
| P1 | Major functionality impaired | 1 hour |
| P2 | Minor functionality impaired | 4 hours |
| P3 | Cosmetic issues | 24 hours |

### Incident Response Playbook

#### 1. Service Down (P0)

```bash
# 1. Acknowledge and communicate
# Notify team via Slack/PagerDuty

# 2. Check status
kubectl get pods -n ai-merchant
kubectl describe pod <pod-name> -n ai-merchant

# 3. Check logs
kubectl logs -n ai-merchant deployment/backend --tail=100

# 4. If needed, restart
kubectl rollout restart deployment/backend -n ai-merchant

# 5. Verify recovery
kubectl rollout status deployment/backend -n ai-merchant
```

#### 2. High Error Rate (P1)

```bash
# Check error logs
kubectl logs -n ai-merchant deployment/backend | grep ERROR

# Check recent deployments
kubectl rollout history deployment/backend -n ai-merchant

# Rollback if needed
kubectl rollout undo deployment/backend -n ai-merchant
```

#### 3. Database Issues (P0/P1)

```bash
# Check connection pool
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"

# Check long-running queries
psql $DATABASE_URL -c "
SELECT pid, state, query_start, query 
FROM pg_stat_activity 
WHERE state = 'active' 
AND query_start < NOW() - INTERVAL '5 minutes';"

# Kill problematic query if needed
psql $DATABASE_URL -c "SELECT pg_terminate_backend(<pid>);"
```

## Capacity Planning

### Current Capacity

| Resource | Current | Max | Threshold |
|----------|---------|-----|-----------|
| Backend Pods | 3 | 10 | 80% CPU |
| Frontend Pods | 2 | 5 | 80% CPU |
| Database | 1 | 1 | 80% CPU |
| Redis | 1 | 1 | 80% Memory |

### Scaling Triggers

```yaml
# Scale up when:
- CPU > 70% for 5 minutes
- Memory > 80% for 5 minutes
- Request latency p95 > 500ms
- Error rate > 1%

# Scale down when:
- CPU < 30% for 10 minutes
- Memory < 40% for 10 minutes
```

## Backup Verification

### Monthly Backup Test

```bash
# 1. Create test environment
docker-compose -f docker-compose.test.yml up -d db

# 2. Download latest backup
aws s3 cp s3://$BACKUP_BUCKET/backups/$(aws s3 ls s3://$BACKUP_BUCKET/backups/ | sort | tail -1 | awk '{print $4}') latest_backup.sql.gz

# 3. Restore to test database
gunzip < latest_backup.sql.gz | docker-compose -f docker-compose.test.yml exec -T db psql -U postgres

# 4. Verify data
docker-compose -f docker-compose.test.yml exec db psql -U postgres -c "SELECT COUNT(*) FROM users;"
docker-compose -f docker-compose.test.yml exec db psql -U postgres -c "SELECT COUNT(*) FROM transactions;"

# 5. Cleanup
docker-compose -f docker-compose.test.yml down -v
rm latest_backup.sql.gz
```

## Security Operations

### Security Scan

```bash
# Scan images for vulnerabilities
docker scan aimerchant/backend:latest
docker scan aimerchant/frontend:latest

# Check dependencies
cd backend && cargo audit
cd ../frontend && npm audit
```

### Log Analysis

```bash
# Failed login attempts
grep "Authentication failed" /var/log/ai-merchant/app.log

# Suspicious activity
grep -E "(SQL injection|XSS|CSRF)" /var/log/nginx/access.log

# Rate limit hits
grep "429" /var/log/nginx/access.log
```

## Performance Tuning

### Database Optimization

```sql
-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM transactions WHERE user_id = 'uuid' ORDER BY created_at DESC LIMIT 10;

-- Check table bloat
SELECT schemaname, tablename, 
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Update statistics
ANALYZE;
```

### Cache Optimization

```bash
# Check Redis memory usage
redis-cli -u $REDIS_URL INFO memory

# Clear old cache entries
redis-cli -u $REDIS_URL --eval scripts/clear_old_cache.lua
```

## Maintenance Procedures

### Monthly Maintenance

1. **Security Updates**
   ```bash
   # Update system packages
   apt update && apt upgrade -y
   
   # Update containers
   docker-compose -f docker-compose.prod.yml pull
   docker-compose -f docker-compose.prod.yml up -d
   ```

2. **Database Maintenance**
   ```sql
   -- Vacuum and analyze
   VACUUM ANALYZE;
   
   -- Reindex
   REINDEX DATABASE ai_merchant;
   ```

3. **Log Rotation**
   ```bash
   # Manual rotation
   logrotate -f /etc/logrotate.conf
   
   # Clean old logs
   find /var/log/ai-merchant -name "*.log.*" -mtime +30 -delete
   ```

## Alert Configuration

### Prometheus Alerts

```yaml
# Critical alerts
- name: InstanceDown
  expr: up == 0
  for: 5m
  severity: critical

- name: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
  for: 5m
  severity: critical
```

### Notification Channels

- **Slack**: #alerts-production
- **PagerDuty**: For P0/P1 incidents
- **Email**: ops-team@aimerchant.app

## Documentation

### Keep Updated

- [ ] Architecture diagrams
- [ ] Network topology
- [ ] Runbooks (this document)
- [ ] Contact information
- [ ] Vendor contacts

## Post-Incident Review

After every P0/P1 incident:

1. Timeline of events
2. Root cause analysis
3. Impact assessment
4. Resolution steps
5. Action items to prevent recurrence
6. Update runbooks if needed

## Contact Information

| Role | Name | Contact | PagerDuty |
|------|------|---------|-----------|
| On-call Engineer | - | - | Yes |
| Engineering Lead | - | - | Yes |
| Product Manager | - | - | No |
| Security Team | - | security@aimerchant.app | Yes |

## Useful Commands Quick Reference

```bash
# Kubernetes
kubectl get pods -n ai-merchant
kubectl logs -f deployment/backend -n ai-merchant
kubectl exec -it deployment/backend -n ai-merchant -- /bin/sh
kubectl top pods -n ai-merchant

# Docker
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs -f
docker system prune -f

# Database
psql $DATABASE_URL
pg_dump $DATABASE_URL > backup.sql
redis-cli -u $REDIS_URL monitor

# SSL
certbot certificates
certbot renew --dry-run
```
