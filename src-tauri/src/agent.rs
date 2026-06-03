//! Exodus Browser — Web agent action space and DOM compression utilities.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent action space - strongly typed enum for web automation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum AgentAction {
    /// Click on an element by selector
    Click {
        selector: String,
    },
    /// Type text into an input field
    Type {
        selector: String,
        text: String,
    },
    /// Scroll the page
    Scroll {
        direction: ScrollDirection,
        distance: i32,
    },
    /// Navigate to a URL
    Navigate {
        url: String,
    },
    /// Wait for a specified time
    Wait {
        ms: u64,
    },
    /// Get page content
    GetContent,
    /// Extract specific element text
    ExtractText {
        selector: String,
    },
    /// Extract all links from page
    ExtractLinks,
    /// Submit a form
    SubmitForm {
        selector: String,
    },
}

/// Scroll direction enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// DOM node representation for compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomNode {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<DomNode>,
    pub is_interactive: bool,
}

/// Compressed DOM representation for agent processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedDom {
    pub url: String,
    pub title: String,
    pub nodes: Vec<DomNode>,
    pub interactive_elements: Vec<String>, // Selectors for clickable elements
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AgentResult {
    pub action: AgentAction,
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}

/// Agent execution context
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgentContext {
    #[allow(dead_code)]
    pub current_url: String,
    #[allow(dead_code)]
    pub dom_snapshot: Option<CompressedDom>,
}

impl AgentContext {
    pub fn new(url: String) -> Self {
        Self {
            current_url: url,
            dom_snapshot: None,
        }
    }

    #[allow(dead_code)]
    pub fn update_dom(&mut self, dom: CompressedDom) {
        let url = dom.url.clone();
        self.dom_snapshot = Some(dom);
        self.current_url = url;
    }
}

/// DOM compression utility
pub struct DomCompressor;

impl DomCompressor {
    /// Compress DOM by removing non-essential elements
    pub fn compress(html: &str, url: String) -> Result<CompressedDom, String> {
        // This is a simplified implementation
        // In production, use a proper HTML parser like scraper or select
        
        let title = Self::extract_title(html);
        let nodes = Self::parse_nodes(html)?;
        let interactive_elements = Self::extract_interactive_selectors(&nodes);
        
        Ok(CompressedDom {
            url,
            title,
            nodes,
            interactive_elements,
        })
    }

    fn extract_title(html: &str) -> String {
        // Simple title extraction
        html.lines()
            .find(|line| line.contains("<title>"))
            .and_then(|line| {
                let start = line.find("<title>")? + 7;
                let end = line.find("</title>")?;
                Some(line[start..end].to_string())
            })
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn parse_nodes(html: &str) -> Result<Vec<DomNode>, String> {
        if html.is_empty() {
            return Ok(Vec::new());
        }

        let document = Html::parse_document(html);
        let body_selector = Selector::parse("body").map_err(|e| e.to_string())?;
        let mut nodes = Vec::new();

        for element in document.select(&body_selector) {
            nodes.push(Self::element_to_node(&element));
        }

        // If no body found, try parsing the whole document
        if nodes.is_empty() {
            for element in document.root_element().children() {
                if let Some(el) = scraper::ElementRef::wrap(element) {
                    nodes.push(Self::element_to_node(&el));
                }
            }
        }

        Ok(nodes)
    }

    fn element_to_node(element: &scraper::ElementRef) -> DomNode {
        let tag = element.value().name().to_string();
        let id = element.value().id().map(|s| s.to_string());
        let classes: Vec<String> = element.value().classes().map(|s| s.to_string()).collect();
        
        let mut attributes = HashMap::new();
        for (name, value) in element.value().attrs() {
            attributes.insert(name.to_string(), value.to_string());
        }

        let text = if tag == "input" || tag == "textarea" {
            element.value().attr("value").map(|s| s.to_string())
        } else {
            let text_content = element.text().collect::<String>();
            if text_content.trim().is_empty() {
                None
            } else {
                Some(text_content.chars().take(500).collect())
            }
        };

        let is_interactive = matches!(
            tag.as_str(),
            "a" | "button" | "input" | "textarea" | "select" | "option"
        ) || attributes.contains_key("onclick")
            || attributes.contains_key("href");

        let children: Vec<DomNode> = element
            .children()
            .filter_map(|child| {
                scraper::ElementRef::wrap(child).map(|el| Self::element_to_node(&el))
            })
            .collect();

        DomNode {
            tag,
            id,
            classes,
            text,
            attributes,
            children,
            is_interactive,
        }
    }

    fn extract_interactive_selectors(nodes: &[DomNode]) -> Vec<String> {
        let mut selectors = Vec::new();
        
        for node in nodes {
            if node.is_interactive {
                if let Some(id) = &node.id {
                    selectors.push(format!("#{}", id));
                }
                for class in &node.classes {
                    selectors.push(format!(".{}", class));
                }
            }
            
            // Recursively check children
            selectors.extend(Self::extract_interactive_selectors(&node.children));
        }
        
        selectors
    }
}

/// Agent action executor
pub struct AgentExecutor {
    #[allow(dead_code)]
    context: AgentContext,
}

impl AgentExecutor {
    pub fn new(initial_url: String) -> Self {
        Self {
            context: AgentContext::new(initial_url),
        }
    }

    /// Execute an action (returns JavaScript code to inject).
    pub fn execute(&self, action: &AgentAction) -> Result<String, String> {
        Self::validate_action(action)?;
        match action {
            AgentAction::Click { selector } => {
                Ok(Self::wrap_selector_action(selector, "el.click();"))
            }
            AgentAction::Type { selector, text } => {
                Ok(Self::wrap_selector_action(
                    selector,
                    &format!("el.value = '{}';", Self::escape_string(text)),
                ))
            }
            AgentAction::Scroll { direction, distance } => {
                let scroll_js = match direction {
                    ScrollDirection::Up => "window.scrollBy(0, -",
                    ScrollDirection::Down => "window.scrollBy(0, ",
                    ScrollDirection::Left => "window.scrollBy(-",
                    ScrollDirection::Right => "window.scrollBy(",
                };
                Ok(format!("{}{});", scroll_js, distance))
            }
            AgentAction::Navigate { url } => {
                Ok(format!("window.location.href = '{}';", Self::escape_string(url)))
            }
            AgentAction::Wait { ms } => {
                Ok(format!("new Promise(resolve => setTimeout(resolve, {}));", ms))
            }
            AgentAction::GetContent => Ok(
                "(function(){ return document.body ? document.body.innerText : ''; })();"
                    .to_string(),
            ),
            AgentAction::ExtractText { selector } => {
                Ok(Self::wrap_selector_action(selector, "return el.innerText;"))
            }
            AgentAction::ExtractLinks => {
                Ok("Array.from(document.querySelectorAll('a')).map(a => a.href).join('\\n');".to_string())
            }
            AgentAction::SubmitForm { selector } => {
                Ok(Self::wrap_selector_action(selector, "el.submit();"))
            }
        }
    }

    /// Reject actions with missing selector or URL before generating JS.
    fn validate_action(action: &AgentAction) -> Result<(), String> {
        match action {
            AgentAction::Click { selector }
            | AgentAction::Type { selector, .. }
            | AgentAction::ExtractText { selector }
            | AgentAction::SubmitForm { selector } => {
                if selector.trim().is_empty() {
                    return Err("Selector cannot be empty".to_string());
                }
            }
            AgentAction::Navigate { url } => {
                if url.trim().is_empty() {
                    return Err("URL cannot be empty".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Run `body` only when `document.querySelector(selector)` finds an element.
    fn wrap_selector_action(selector: &str, body: &str) -> String {
        let sel = Self::escape_selector(selector);
        let label = Self::escape_string(selector);
        format!(
            "(function(){{ const el = document.querySelector('{sel}'); if (!el) throw new Error('Element not found: {label}'); {body} }})();",
            sel = sel,
            label = label,
            body = body
        )
    }

    fn escape_selector(selector: &str) -> String {
        selector.replace('\\', "\\\\").replace('\'', "\\'")
    }

    fn escape_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_action_serialization() {
        let action = AgentAction::Click {
            selector: "#submit-button".to_string(),
        };
        
        let json = serde_json::to_string(&action).expect("Failed to serialize action");
        assert!(json.contains("Click"));
        assert!(json.contains("submit-button"));
    }

    #[test]
    fn execute_rejects_empty_selector() {
        let executor = AgentExecutor::new("https://example.com".into());
        let action = AgentAction::Click {
            selector: "  ".into(),
        };
        let err = executor.execute(&action).unwrap_err();
        assert!(err.contains("Selector"));
    }

    #[test]
    fn execute_click_includes_null_guard() {
        let executor = AgentExecutor::new("https://example.com".into());
        let action = AgentAction::Click {
            selector: "#btn".into(),
        };
        let js = executor.execute(&action).expect("Failed to execute action");
        assert!(js.contains("if (!el)"));
        assert!(js.contains("el.click()"));
    }

    #[test]
    fn test_dom_compression() {
        let html = "<html><head><title>Test</title></head><body><p>Hello</p></body></html>";
        let result = DomCompressor::compress(html, "http://example.com".to_string());
        
        assert!(result.is_ok());
        let compressed = result.expect("Failed to compress DOM");
        assert_eq!(compressed.title, "Test");
        assert_eq!(compressed.url, "http://example.com");
    }
}
