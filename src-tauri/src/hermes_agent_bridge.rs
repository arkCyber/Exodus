//! Hermes ↔ Web Agent bridge — map commands/tasks to `AgentAction` JSON for DOM execution.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::agent::{AgentAction, AgentExecutor, ScrollDirection};

/// Planned outcome for the browser agent panel (DOM action, Allama ask, or none).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesActionPlan {
    /// `dom` | `ask` | `none`
    pub kind: String,
    pub action_json: Option<String>,
    pub javascript: Option<String>,
    pub ask_prompt: Option<String>,
    pub message: Option<String>,
}

/// Plan a browser agent command (natural language or raw JSON action).
pub fn plan_agent_command(command: &str, current_url: &str) -> Result<HermesActionPlan, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Empty command".into());
    }

    if let Ok(action) = serde_json::from_str::<AgentAction>(trimmed) {
        return plan_from_action(&action, current_url);
    }

    let lower = trimmed.to_ascii_lowercase();
    if let Some(prompt) = parse_ask_prompt(trimmed) {
        return Ok(HermesActionPlan {
            kind: "ask".into(),
            action_json: None,
            javascript: None,
            ask_prompt: Some(prompt),
            message: Some("Routed to Hermes page analysis".into()),
        });
    }

    let action = if lower.contains("scroll") && lower.contains("down") {
        AgentAction::Scroll {
            direction: ScrollDirection::Down,
            distance: 500,
        }
    } else if lower.contains("scroll") && lower.contains("up") {
        AgentAction::Scroll {
            direction: ScrollDirection::Up,
            distance: 500,
        }
    } else if lower.contains("link") {
        AgentAction::ExtractLinks
    } else if lower.contains("content") || lower.contains("text") {
        AgentAction::GetContent
    } else if lower.starts_with("http://") || lower.starts_with("https://") {
        AgentAction::Navigate {
            url: trimmed.to_string(),
        }
    } else if lower.starts_with("navigate ") {
        let url = trimmed["navigate ".len()..].trim();
        if url.is_empty() {
            return Err("Navigate requires a URL".into());
        }
        AgentAction::Navigate {
            url: url.to_string(),
        }
    } else {
        return Ok(HermesActionPlan {
            kind: "none".into(),
            action_json: None,
            javascript: None,
            ask_prompt: None,
            message: Some(
                "Use JSON, scroll up/down, links, content, ask: question, or a URL".into(),
            ),
        });
    };

    plan_from_action(&action, current_url)
}

/// Build a plan from task metadata (used by Hermes task executors).
pub fn plan_from_task_metadata(
    task_type: &str,
    description: &str,
    metadata: &HashMap<String, String>,
    current_url: &str,
) -> Result<HermesActionPlan, String> {
    if let Some(json) = metadata.get("action_json") {
        let action: AgentAction = serde_json::from_str(json)
            .map_err(|e| format!("Invalid action_json: {e}"))?;
        return plan_from_action(&action, current_url);
    }

    match task_type {
        "Navigation" => {
            let url = metadata
                .get("url")
                .cloned()
                .filter(|u| !u.is_empty())
                .ok_or_else(|| "Navigation task requires metadata.url".to_string())?;
            plan_from_action(
                &AgentAction::Navigate { url },
                current_url,
            )
        }
        "DataExtraction" => {
            let extract = metadata
                .get("extract")
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "content".to_string());
            let action = match extract.as_str() {
                "links" => AgentAction::ExtractLinks,
                "text" | "content" => AgentAction::GetContent,
                other if !other.is_empty() => AgentAction::ExtractText {
                    selector: other.to_string(),
                },
                _ => AgentAction::GetContent,
            };
            plan_from_action(&action, current_url)
        }
        "FormFill" => {
            let selector = metadata
                .get("selector")
                .ok_or("FormFill requires metadata.selector")?
                .clone();
            let text = metadata
                .get("text")
                .cloned()
                .unwrap_or_else(|| description.to_string());
            plan_from_action(
                &AgentAction::Type { selector, text },
                current_url,
            )
        }
        "Automation" => {
            if let Some(cmd) = metadata.get("command") {
                plan_agent_command(cmd, current_url)
            } else {
                plan_agent_command(description, current_url)
            }
        }
        _ => Ok(HermesActionPlan {
            kind: "none".into(),
            action_json: None,
            javascript: None,
            ask_prompt: None,
            message: None,
        }),
    }
}

fn plan_from_action(action: &AgentAction, current_url: &str) -> Result<HermesActionPlan, String> {
    let action_json = serde_json::to_string(action).map_err(|e| e.to_string())?;
    let executor = AgentExecutor::new(current_url.to_string());
    let javascript = executor.execute(action).ok();
    Ok(HermesActionPlan {
        kind: "dom".into(),
        action_json: Some(action_json),
        javascript,
        ask_prompt: None,
        message: Some("DOM action planned".into()),
    })
}

/// Parse a JSON array of command strings from Allama output (handles markdown fences).
pub fn parse_command_list_from_llm(text: &str) -> Result<Vec<String>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Empty LLM response".into());
    }
    if let Ok(list) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Ok(list);
    }
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(arr) = val.get("steps").and_then(|s| s.as_array()) {
            return json_array_to_commands(arr);
        }
        if let Some(arr) = val.as_array() {
            return json_array_to_commands(arr);
        }
    }
    let start = trimmed.find('[').ok_or("No JSON array in LLM response")?;
    let end = trimmed.rfind(']').ok_or("No JSON array in LLM response")?;
    let slice = &trimmed[start..=end];
    let list: Vec<String> = serde_json::from_str(slice)
        .map_err(|e| format!("Failed to parse command list: {e}"))?;
    Ok(list)
}

fn json_array_to_commands(arr: &[serde_json::Value]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for v in arr {
        if let Some(s) = v.as_str() {
            let t = s.trim();
            if !t.is_empty() {
                out.push(t.to_string());
            }
        }
    }
    if out.is_empty() {
        return Err("LLM returned empty step list".into());
    }
    Ok(out)
}

/// Split `scroll down then get links` style goals without LLM.
pub fn split_goal_into_commands(goal: &str) -> Vec<String> {
    goal.split(" then ")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn parse_ask_prompt(command: &str) -> Option<String> {
    let re = command.trim();
    if let Some(rest) = re.strip_prefix("ask:") {
        let p = rest.trim();
        if !p.is_empty() {
            return Some(p.to_string());
        }
    }
    if let Some(rest) = re.strip_prefix("ask ") {
        let p = rest.trim();
        if !p.is_empty() {
            return Some(p.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_scroll_down() {
        let plan = plan_agent_command("scroll down", "https://example.com")
            .expect("plan");
        assert_eq!(plan.kind, "dom");
        assert!(plan.action_json.as_ref().unwrap().contains("Scroll"));
    }

    #[test]
    fn plan_ask_prefix() {
        let plan = plan_agent_command("ask: summarize", "https://example.com")
            .expect("plan");
        assert_eq!(plan.kind, "ask");
        assert_eq!(plan.ask_prompt.as_deref(), Some("summarize"));
    }

    #[test]
    fn parse_command_list_json() {
        let list = parse_command_list_from_llm(r#"["scroll down", "links"]"#).expect("parse");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "scroll down");
    }

    #[test]
    fn split_goal_then() {
        let parts = split_goal_into_commands("scroll down then extract links");
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn plan_navigation_metadata() {
        let mut meta = HashMap::new();
        meta.insert("url".to_string(), "https://example.org".to_string());
        let plan = plan_from_task_metadata(
            "Navigation",
            "go",
            &meta,
            "https://example.com",
        )
        .expect("plan");
        assert_eq!(plan.kind, "dom");
        assert!(plan.javascript.as_ref().unwrap().contains("example.org"));
    }
}
