#!/bin/bash
# ASI Database Backup Script
# Usage: ./scripts/backup.sh [output_dir]
set -euo pipefail

DB_PATH="${DATABASE_URL:-asi.db}"
BACKUP_DIR="${1:-./backups}"
TIMESTAMP=$(date -u +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/asi_${TIMESTAMP}.db"

mkdir -p "$BACKUP_DIR"

echo "Backing up ${DB_PATH} to ${BACKUP_FILE}..."

# Use sqlite3 backup API (safe for live databases)
sqlite3 "$DB_PATH" ".backup '${BACKUP_FILE}'"

# Compress
gzip -f "$BACKUP_FILE"
echo "Backup complete: ${BACKUP_FILE}.gz ($(wc -c < "${BACKUP_FILE}.gz") bytes)"

# Keep only last 7 backups
ls -t "${BACKUP_DIR}"/asi_*.db.gz 2>/dev/null | tail -n +8 | xargs rm -f 2>/dev/null || true

echo "Retained last 7 backups."
