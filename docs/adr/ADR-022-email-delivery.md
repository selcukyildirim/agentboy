# ADR-022: Email Delivery Provider

## Status

Proposed

## Context

Email verification and password reset.

## Decision

**SendGrid** or **AWS SES** (to be confirmed).

## Alternatives

- **Mailgun**: Good but cost
- **SMTP direct**: More control but complex

## Consequences

- High deliverability
- Template support
- Analytics
- API-based