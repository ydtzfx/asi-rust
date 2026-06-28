---
name: migrate
description: Database migration management — create, run, verify, rollback
disable-model-invocation: true
---

# Database Migrations

Manage SQLite database migrations for the ASI project.

## Create a new migration
```bash
# Add a timestamped migration file
TIMESTAMP=$(date -u +%Y%m%d%H%M%S)
touch migrations/${TIMESTAMP}_description.sql
# Add SQL, then rebuild — migrations run at server startup
```

## Verify migration status
```bash
sqlite3 asi.db "SELECT * FROM _migrations ORDER BY id;"
```

## Backup before migration
```bash
./scripts/backup.sh
```

## Rollback (manual)
```bash
# Restore from backup
./scripts/restore.sh backups/asi_YYYYMMDD_HHMMSS.db.gz
```

## Migration checklist
- [ ] Backup created before migration
- [ ] SQL tested in sqlite3 CLI first
- [ ] Rollback plan documented
- [ ] `cargo test -p asi-db` passes after migration
