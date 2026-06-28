#!/bin/bash
# ASI Database Restore Script
# Usage: ./scripts/restore.sh <backup_file.gz>
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup_file.gz>"
    echo "Available backups:"
    ls -t ./backups/asi_*.db.gz 2>/dev/null || echo "  (none)"
    exit 1
fi

BACKUP_FILE="$1"
DB_PATH="${DATABASE_URL:-asi.db}"

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Restoring ${BACKUP_FILE} to ${DB_PATH}..."

# Backup current DB first (safety net)
if [ -f "$DB_PATH" ]; then
    SAFETY_BACKUP="${DB_PATH}.pre_restore_$(date -u +%Y%m%d_%H%M%S)"
    cp "$DB_PATH" "$SAFETY_BACKUP"
    echo "  Safety backup: ${SAFETY_BACKUP}"
fi

# Decompress and restore
gunzip -c "$BACKUP_FILE" > "${DB_PATH}.restore"
mv "${DB_PATH}.restore" "$DB_PATH"

echo "Restore complete. Restart the server to apply."
