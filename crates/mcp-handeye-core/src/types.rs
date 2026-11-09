//! Type definitions with validation

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Input parameters for the add operation
///
/// This demonstrates type safety with automatic schema generation and validation:
/// - `schemars::JsonSchema` automatically generates detailed JSON schema for MCP clients
/// - `validator::Validate` provides runtime validation with custom constraints
/// - `serde` handles JSON serialization/deserialization
/// - `deny_unknown_fields` rejects any extra fields for security
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct AddInput {
    /// First number to add (must be between -1,000,000 and 1,000,000)
    #[validate(range(min = -1000000.0, max = 1000000.0))]
    #[schemars(description = "First number in the addition operation")]
    pub a: f64,

    /// Second number to add (must be between -1,000,000 and 1,000,000)
    #[validate(range(min = -1000000.0, max = 1000000.0))]
    #[schemars(description = "Second number in the addition operation")]
    pub b: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let input = AddInput { a: 5.0, b: 3.0 };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_out_of_range() {
        let input = AddInput {
            a: 2000000.0,
            b: 3.0,
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_negative_numbers() {
        let input = AddInput { a: -5.0, b: -3.0 };
        assert!(input.validate().is_ok());
    }
}
