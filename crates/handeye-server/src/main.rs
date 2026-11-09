//! Handeye Server Binary

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = mcp_handeye_core::build_handeye_server()?;
    server_common::run_http(server, "handeye", "1.0.0").await
}
