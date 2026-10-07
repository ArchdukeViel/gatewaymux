# Rule: Database Migrations

## Purpose
Govern database schema evolution, integrity, and rollback capability for local and sync persistence.

## Invariants
1. **Migrations Are Mandatory**: Once local SQLite persistence or sync database schemas are introduced, all schema changes MUST be implemented as versioned migration files.
2. **Reversible & Idempotent**: Every migration must be tested for both forward execution (`up`) and backward rollback (`down`). Migrations must be safely idempotent where feasible.
3. **Naming Convention**: Use monotonically increasing numeric prefixes and descriptive snake_case names: `0001_initial_schema.sql`, `0002_add_quota_buckets.sql`.
4. **No Destructive Drops**: Migrations must avoid destructive schema operations that destroy user configuration without an automated, verified data migration step.
5. **No Direct Schema Mutation**: Application code must never execute ad-hoc `CREATE TABLE` or `ALTER TABLE` statements outside the migration manager.
