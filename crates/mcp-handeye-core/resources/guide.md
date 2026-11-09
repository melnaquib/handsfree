# Handeye Server Guide

## Overview

The handeye server provides tools for mathematical operations.

## Available Tools

### add

Adds two numbers together with validation.

**Parameters:**
- `a` (number): First number (range: -1,000,000 to 1,000,000)
- `b` (number): Second number (range: -1,000,000 to 1,000,000)

**Example:**
```json
{
  "a": 5,
  "b": 3
}
```

**Response:**
```json
{
  "result": 8,
  "operation": "5 + 3 = 8"
}
```

## Validation

All inputs are validated:
- Numeric ranges prevent overflow/DoS attacks
- Unknown fields are rejected
- Type safety enforced at compile time

## Error Handling

The server returns structured errors:
- Validation errors include field-level details
- Type errors indicate schema mismatches
- Server errors are logged with request IDs
