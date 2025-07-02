# Testing

Cria includes both unit tests and integration tests.

## Running Tests

### Unit Tests Only (for CI/NixOS builds)
```bash
# Skip integration tests that require a Vikunja server
SKIP_INTEGRATION_TESTS=1 cargo test
```

### All Tests (for development)
```bash
# Run all tests including integration tests
# Requires a running Vikunja server and valid credentials
export VIKUNJA_URL="http://localhost:3456"
export VIKUNJA_TOKEN="your-api-token"
export VIKUNJA_PROJECT_ID="1"
cargo test
```

## Integration Tests

Integration tests in `tests/api_*.rs` require:
- A running Vikunja server
- Valid API credentials set via environment variables
- Network access

These tests will automatically skip if:
- No `VIKUNJA_TOKEN` environment variable is set
- `SKIP_INTEGRATION_TESTS` environment variable is set
- Network access is unavailable

This ensures builds work in sandboxed environments like NixOS.

## Test Categories

- **Unit Tests**: Parser logic, utilities, data structures
- **Integration Tests**: API operations against real Vikunja server  
- **UI Tests**: Modal and picker event handling logic
