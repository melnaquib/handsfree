//! Handeye MCP Server Core
//!
//! This crate contains the business logic for the handeye server.

mod types;

use pmcp::{Server, TypedTool};
use pmcp::types::capabilities::ServerCapabilities;
use serde_json::json;
use validator::Validate;
use types::*;

/// Build the Handeye server
pub fn build_handeye_server() -> pmcp::Result<Server> {
    Server::builder()
        .name("handeye")
        .version("1.0.0")
        .capabilities(ServerCapabilities::tools_only())
        .tool(
            "add",
            TypedTool::new("add", |input: AddInput, _extra| {
                Box::pin(async move {
                    // Validate using validator crate
                    input.validate()
                        .map_err(|e| pmcp::Error::validation(format!("Validation failed: {}", e)))?;

                    // Perform calculation
                    let result = input.a + input.b;

                    Ok(json!({
                        "result": result,
                        "operation": format!("{} + {} = {}", input.a, input.b, result)
                    }))
                })
            })
            .with_description("Add two numbers together with range validation"),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_builds() {
        let server = build_handeye_server();
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_add_validation() {
        let input = AddInput { a: 5.0, b: 3.0 };
        assert!(input.validate().is_ok());

        // Test out of range
        let input = AddInput { a: 2000000.0, b: 3.0 };
        assert!(input.validate().is_err());
    }

    #[tokio::test]
    async fn test_add_logic() {
        let input = AddInput { a: 5.0, b: 3.0 };
        assert_eq!(input.a + input.b, 8.0);

        let input = AddInput { a: -5.0, b: 3.0 };
        assert_eq!(input.a + input.b, -2.0);
    }
}
