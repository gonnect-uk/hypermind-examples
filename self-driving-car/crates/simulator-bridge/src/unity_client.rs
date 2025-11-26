//! Unity simulator Socket.IO client
//!
//! Note: This is a placeholder implementation. Full Socket.IO integration
//! will be added when we test with actual Unity simulator.

use anyhow::Result;
use serde_json::Value;

/// Unity simulator client (Socket.IO on port 4567)
pub struct UnityClient {
    host: String,
    port: u16,
}

impl UnityClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Connect to Unity simulator
    pub async fn connect(&self) -> Result<()> {
        // TODO: Implement Socket.IO connection
        // For now, this is a placeholder
        tracing::info!("Connecting to Unity at {}:{}", self.host, self.port);
        Ok(())
    }

    /// Send control command to Unity
    pub async fn send_control(&self, command: Value) -> Result<()> {
        tracing::debug!("Sending control: {}", command);
        // TODO: Implement Socket.IO emit
        Ok(())
    }

    /// Listen for telemetry from Unity
    pub async fn listen_telemetry<F>(&self, _callback: F) -> Result<()>
    where
        F: Fn(Value) + Send + 'static,
    {
        // TODO: Implement Socket.IO listener
        tracing::info!("Listening for telemetry...");
        Ok(())
    }
}

impl Default for UnityClient {
    fn default() -> Self {
        Self::new("127.0.0.1", 4567)
    }
}
