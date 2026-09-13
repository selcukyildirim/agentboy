# ADR-014: Queue/Event Technology

## Status

Proposed

## Context

Paid server needs async task processing.

## Decision

PostgreSQL-based queue or RabbitMQ (to be confirmed).

## Alternatives

- **NATS**: Lightweight, fast
- **Kafka**: Heavy but powerful

## Consequences

- Reliable message delivery
- Retry support
- Dead letter queue
- Priority queues