use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::{Result, anyhow};
use tokio::time::sleep;
use tracing::{info, warn, error};

/// ChromaDB server manager that handles starting and stopping the ChromaDB server
#[derive(Debug)]
pub struct ChromaDbServerManager {
    port: u16,
    host: String,
    process: Arc<Mutex<Option<Child>>>,
    data_path: Option<String>,
}

impl ChromaDbServerManager {
    /// Create a new ChromaDB server manager
    pub fn new() -> Self {
        Self {
            port: 8465, // Using custom port instead of default 8000
            host: "localhost".to_string(),
            process: Arc::new(Mutex::new(None)),
            data_path: None,
        }
    }

    /// Create a new ChromaDB server manager with custom configuration
    pub fn with_config(port: u16, host: String, data_path: Option<String>) -> Self {
        Self {
            port,
            host,
            process: Arc::new(Mutex::new(None)),
            data_path,
        }
    }

    /// Get the base URL for the ChromaDB server
    pub fn get_base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Check if ChromaDB is installed and available
    pub fn check_chromadb_available(&self) -> Result<()> {
        // Check if chromadb command is available
        match which::which("chroma") {
            Ok(path) => {
                info!("Found ChromaDB at: {:?}", path);
                Ok(())
            }
            Err(_) => {
                // Try python -m chromadb as alternative
                match Command::new("python3")
                    .args(&["-m", "chromadb", "--help"])
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        info!("Found ChromaDB via python module");
                        Ok(())
                    }
                    _ => {
                        error!("ChromaDB not found. Please install ChromaDB: pip install chromadb");
                        Err(anyhow!("ChromaDB not installed"))
                    }
                }
            }
        }
    }

    /// Start the ChromaDB server
    pub async fn start_server(&self) -> Result<()> {
        // Check if already running
        if self.is_server_running().await {
            info!("ChromaDB server already running on port {}", self.port);
            return Ok(());
        }

        // Check if ChromaDB is available
        self.check_chromadb_available()?;

        info!("Starting ChromaDB server on {}:{}", self.host, self.port);

        // Prepare command arguments
        let mut args = vec![
            "run".to_string(),
            "--host".to_string(),
            self.host.clone(),
            "--port".to_string(),
            self.port.to_string(),
        ];

        // Add data path if specified
        if let Some(data_path) = &self.data_path {
            args.push("--path".to_string());
            args.push(data_path.clone());
        }

        // Always use .venv/bin/chroma for startup
        let venv_chroma = std::env::current_dir()
            .map(|dir| dir.join(".venv/bin/chroma"))
            .unwrap_or_else(|_| std::path::PathBuf::from("chroma"));
        let child = Command::new(&venv_chroma)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start ChromaDB server: {}", e))?;

        // Store the process handle
        {
            let mut process_guard = self.process.lock().unwrap();
            *process_guard = Some(child);
        }

        // Wait for server to start up (ChromaDB can be slow on first start)
        let max_attempts = 60;
        let mut attempts = 0;

        while attempts < max_attempts {
            if self.is_server_running().await {
                info!("ChromaDB server started successfully on {}", self.get_base_url());
                return Ok(());
            }

            attempts += 1;
            sleep(Duration::from_secs(1)).await;
        }

        // If we get here, server failed to start - capture error output
        let error_output = {
            let mut process_guard = self.process.lock().unwrap();
            if let Some(ref mut child) = process_guard.as_mut() {
                if let Some(stderr) = child.stderr.take() {
                    use std::io::Read;
                    let mut error_msg = String::new();
                    let mut reader = std::io::BufReader::new(stderr);
                    let _ = reader.read_to_string(&mut error_msg);
                    if !error_msg.is_empty() {
                        Some(error_msg)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        self.stop_server().await?;
        
        let error_msg = if let Some(stderr_output) = error_output {
            format!("ChromaDB server failed to start within {} seconds. Error output: {}", max_attempts, stderr_output)
        } else {
            format!("ChromaDB server failed to start within {} seconds", max_attempts)
        };
        
        Err(anyhow!(error_msg))
    }

    /// Check if the ChromaDB server is running
    pub async fn is_server_running(&self) -> bool {
        let client = reqwest::Client::new();
        let health_url = format!("{}/api/v2/heartbeat", self.get_base_url());

        match client
            .get(&health_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Stop the ChromaDB server
    pub async fn stop_server(&self) -> Result<()> {
        // Extract the child process from the mutex and drop the guard
        let child_opt = {
            let mut process_guard = self.process.lock().unwrap();
            process_guard.take()
        }; // MutexGuard is dropped here
        
        if let Some(mut child) = child_opt {
            info!("Stopping ChromaDB server...");
            
            // Try graceful shutdown first
            if let Err(e) = child.terminate() {
                warn!("Failed to terminate ChromaDB process gracefully: {}", e);
                
                // Force kill if graceful shutdown fails
                if let Err(e) = child.kill() {
                    error!("Failed to kill ChromaDB process: {}", e);
                    return Err(anyhow!("Failed to stop ChromaDB server: {}", e));
                }
            }

            // Wait for process to exit
            match child.wait() {
                Ok(status) => {
                    info!("ChromaDB server stopped with status: {}", status);
                }
                Err(e) => {
                    warn!("Error waiting for ChromaDB process to exit: {}", e);
                }
            }
        }

        // Verify server is actually stopped
        let max_attempts = 10;
        let mut attempts = 0;

        while attempts < max_attempts && self.is_server_running().await {
            attempts += 1;
            sleep(Duration::from_millis(500)).await;
        }

        if self.is_server_running().await {
            warn!("ChromaDB server may still be running after stop attempt");
        } else {
            info!("ChromaDB server stopped successfully");
        }

        Ok(())
    }

    /// Get server status information
    pub async fn get_server_status(&self) -> ServerStatus {
        let is_running = self.is_server_running().await;
        let has_process = {
            let process_guard = self.process.lock().unwrap();
            process_guard.is_some()
        };

        ServerStatus {
            is_running,
            has_process,
            base_url: self.get_base_url(),
            port: self.port,
        }
    }
}

impl Drop for ChromaDbServerManager {
    fn drop(&mut self) {
        // Attempt to stop server on drop
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            if let Err(e) = rt.block_on(self.stop_server()) {
                error!("Failed to stop ChromaDB server during cleanup: {}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub is_running: bool,
    pub has_process: bool,
    pub base_url: String,
    pub port: u16,
}

// Extension trait for Child process to add terminate method
trait ChildExt {
    fn terminate(&mut self) -> std::io::Result<()>;
}

impl ChildExt for Child {
    fn terminate(&mut self) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            // Send SIGTERM for graceful shutdown
            unsafe {
                libc::kill(self.id() as i32, libc::SIGTERM);
            }
            Ok(())
        }

        #[cfg(windows)]
        {
            // On Windows, just kill the process
            self.kill()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_manager_creation() {
        let manager = ChromaDbServerManager::new();
        assert_eq!(manager.port, 8465);
        assert_eq!(manager.get_base_url(), "http://localhost:8465");
    }

    #[tokio::test]
    async fn test_server_status() {
        let manager = ChromaDbServerManager::new();
        let status = manager.get_server_status().await;
        assert_eq!(status.port, 8465);
        assert_eq!(status.base_url, "http://localhost:8465");
    }
}
