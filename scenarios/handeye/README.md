# Test Scenarios

This directory contains test scenarios for your MCP server.

## Quick Start

```bash
# 1. Start your server in another terminal
cargo pmcp dev --server NAME

# 2. Generate test scenarios
cargo pmcp test --server NAME --generate-scenarios

# 3. Run tests
cargo pmcp test --server NAME --detailed
```

## ⚠️ Known Limitation: Tool Assertion Paths

**You will see assertion failures for tool calls in the generated scenarios.** This is expected behavior in Phase 1.

### Why Tool Assertions Fail

MCP wraps tool responses in a nested structure:

```json
{
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"result\":357,\"operation\":\"123 + 234 = 357\"}"
    }]
  }
}
```

The generated scenarios assert on `result` directly, but the actual value is in `content[0].text` (as a JSON string).

### What To Do

**Option 1: Use `contains` assertions (Recommended for Phase 1)**

Edit your `generated.yaml` and change tool assertions to use `contains`:

```yaml
- name: 'Test tool: add (123 + 234 = 357)'
  operation:
    type: tool_call
    tool: add
    arguments:
      a: 123
      b: 234
  continue_on_failure: false  # Change to false once fixed
  assertions:
    - type: success
    - type: contains
      path: "content[0].text"
      value: "357"  # Just check result is in the response
```

**Option 2: Accept the limitation for now**

The scenarios are marked with `continue_on_failure: true`, so tests will pass overall even if individual tool assertions fail. This allows you to validate server connectivity and basic functionality while the assertion system is improved.

## Generating Scenarios

```bash
cargo pmcp test --server NAME --generate-scenarios
```

This will:
1. Discover all tools, prompts, and resources from your running server
2. Generate smart test cases with meaningful values (e.g., add(123, 234) = 357)
3. Create assertions to verify expected results
4. Save scenarios to `generated.yaml`

## Running Tests

```bash
# Run all scenarios
cargo pmcp test --server NAME

# Run with detailed output (see what's failing)
cargo pmcp test --server NAME --detailed
```

## Customizing Scenarios

Edit the generated `generated.yaml` file to:
- Add more test cases
- Customize test values
- Fix assertion paths (see above)
- Test edge cases and error conditions
- Add validation for error scenarios

## Scenario Format

See https://docs.example.com/mcp-tester for full documentation on scenario format.
