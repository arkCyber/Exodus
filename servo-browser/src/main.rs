//! Exodus Browser - Pure Rust Browser Implementation
//! Using wry WebView (same rendering as Tauri but pure Rust architecture)

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wry::{WebView, WebViewBuilder};
use tokio::sync::Mutex;

mod rag;
mod agent;
mod sidecar;

use rag::{RagDatabase, create_webpage};
use agent::{AgentExecutor, AgentAction, DomCompressor};
use sidecar::SidecarManager;

struct ExodusBrowser {
    webview: Option<WebView>,
    rag_db: Arc<Mutex<RagDatabase>>,
    sidecar: SidecarManager,
    current_url: Arc<Mutex<String>>,
}

impl ExodusBrowser {
    fn new(rag_db: Arc<Mutex<RagDatabase>>, sidecar: SidecarManager) -> Self {
        Self {
            webview: None,
            rag_db,
            sidecar,
            current_url: Arc::new(Mutex::new("https://news.ycombinator.com".to_string())),
        }
    }

    fn navigate(&self, url: String) {
        if let Some(webview) = &self.webview {
            if let Err(e) = webview.load_url(&url) {
                eprintln!("[Exodus] Failed to navigate: {}", e);
            }
        }
        *self.current_url.blocking_lock() = url;
    }

    async fn capture_page(&self, url: String) {
        // Fetch page content for RAG indexing
        let url_clone = url.clone();
        println!("[Exodus] Capturing page for RAG: {}", url_clone);
        
        match reqwest::get(&url).await {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("[Exodus] HTTP error: {}", response.status());
                    return;
                }
                
                match response.text().await {
                    Ok(html) => {
                        let title = Self::extract_title(&html);
                        let content = html.chars().take(10000).collect::<String>();
                        let page = create_webpage(url, title, content);
                        
                        match self.rag_db.lock().await.store_page(page).await {
                            Ok(_) => println!("[Exodus] Page captured for RAG: {}", url_clone),
                            Err(e) => eprintln!("[Exodus] Failed to store page: {}", e),
                        }
                    }
                    Err(e) => eprintln!("[Exodus] Failed to get page content: {}", e),
                }
            }
            Err(e) => eprintln!("[Exodus] Failed to fetch page: {}", e),
        }
    }

    fn extract_title(html: &str) -> String {
        html.lines()
            .find(|line| line.contains("<title>"))
            .and_then(|line| {
                let start = line.find("<title>")? + 7;
                let end = line.find("</title>")?;
                Some(line[start..end].to_string())
            })
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

#[tokio::main]
async fn main() -> wry::Result<()> {
    env_logger::init();
    
    println!("⛵ Exodus Browser - Pure Rust Implementation");
    println!("Using wry WebView for rendering (system WebView components)");
    
    // Initialize RAG database
    let rag_db = Arc::new(Mutex::new(
        RagDatabase::new().expect("Failed to initialize RAG database")
    ));
    
    // Initialize Sidecar manager
    let sidecar = SidecarManager::new();
    
    // Try to spawn sidecar
    if let Err(e) = sidecar.spawn(&["--port", "11434"]) {
        println!("[Exodus] Sidecar not available: {}", e);
        println!("[Exodus] Continuing without AI features");
    }
    
    let browser = Arc::new(Mutex::new(ExodusBrowser::new(rag_db, sidecar)));
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Exodus Browser - Pure Rust")
        .with_inner_size(winit::dpi::PhysicalSize::new(1400, 900))
        .build(&event_loop)
        .unwrap();
    
    let webview = WebViewBuilder::new(&window)
        .with_url("https://news.ycombinator.com")
        .build()?;
    
    browser.lock().await.webview = Some(webview);
    
    println!("[Exodus] Browser started. RAG capture and Agent features ready for integration.");
    
    // Initial page capture
    let browser_clone = browser.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let url = browser_clone.lock().await.current_url.lock().await.clone();
        browser_clone.lock().await.capture_page(url).await;
    });
    
    event_loop.run(move |event, _control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            std::process::exit(0);
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(physical_size),
            ..
        } => {
            println!("Window resized: {:?}", physical_size);
        }
        _ => {}
    });
    
    Ok(())
}
