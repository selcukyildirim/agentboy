# ADR-013: Distributed Cache

## Status

Proposed

## Context

Paid server needs distributed caching.

## Decision

Redis (to be confirmed during PHASE-11).

## Alternatives

- Memcached: Simpler but less features
- etcd: More for configuration

## Consequences

- In-memory performance
- Persistence option
- Pub/Sub for invalidation
- Cluster support