// Sidecar process management for exodus-core
// This module handles spawning and managing the exodus-core inference engine

use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};

pub struct SidecarManager {
    child: Arc<Mutex<Option<Child>>>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Spawn the exodus-core sidecar process
    pub fn spawn(&self, args: &[&str]) -> Result<(), String> {
        let binary_name = "exodus-core";
        
        // Try to find the binary in common locations
        let binary_paths = vec![
            format!("./binaries/{}", binary_name),
            format!("./src-tauri/binaries/{}", binary_name),
            binary_name.to_string(),
        ];
        
        let mut child = None;
        for path in binary_paths {
            match Command::new(&path)
                .args(args)
                .spawn()
            {
                Ok(spawned) => {
                    child = Some(spawned);
                    println!("[Exodus] Sidecar spawned from: {}", path);
                    break;
                }
                Err(_) => continue,
            }
        }
        
        match child {
            Some(c) => {
                let child_arc = Arc::clone(&self.child);
                *self.child.lock().unwrap() = Some(c);
                
                // Note: For simplicity, we skip stdout/stderr monitoring for now
                // In a production implementation, you would need to use channels
                // or a different approach to share stdout/stderr across threads
                println!("[Exodus] Sidecar spawned successfully");
                
                Ok(())
            }
            None => Err(format!(
                "Failed to find exodus-core binary. Please ensure it's in binaries/ or PATH"
            )),
        }
    }
    
    /// Check if the sidecar is running
    pub fn is_running(&self) -> bool {
        let child_opt = self.child.lock().unwrap();
        child_opt.is_some()
    }
    
    /// Terminate the sidecar process
    pub fn terminate(&self) -> Result<(), String> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            child.kill().map_err(|e| format!("Failed to kill sidecar: {}", e))?;
            println!("[Exodus] Sidecar terminated");
            Ok(())
        } else {
            Err("No sidecar process running".to_string())
        }
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        // Automatically terminate sidecar when manager is dropped
        if let Err(e) = self.terminate() {
            eprintln!("[Exodus] Failed to terminate sidecar on drop: {}", e);
        }
    }
}
