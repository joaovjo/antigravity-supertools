use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HookInput {
    pub hook_event_name: Option<String>,
    pub transcript_path: Option<String>,
    pub session_id: Option<String>,
    pub permission_mode: Option<String>,
    pub turn_id: Option<String>,
    pub stop_hook_active: Option<bool>,
    pub tool_input: Option<ToolInput>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToolInput {
    pub plan: Option<String>,
    pub plan_filename: Option<String>,
    pub plan_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PermissionUpdate {
    pub r#type: String,
    pub mode: String,
    pub destination: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ClaudeDecision {
    pub behavior: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedPermissions")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HookSpecificOutput {
    #[serde(rename = "hookEventName")]
    pub hook_event_name: String,
    pub decision: ClaudeDecision,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ClaudeOutput {
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: HookSpecificOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GeminiOutput {
    #[serde(rename = "systemMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub fn parse_input(json_str: &str) -> Result<HookInput, serde_json::Error> {
    serde_json::from_str(json_str)
}

pub fn build_claude_approval(permission_mode: Option<String>) -> ClaudeOutput {
    let updated_permissions = permission_mode.map(|mode| {
        vec![PermissionUpdate {
            r#type: "setMode".to_string(),
            mode,
            destination: "session".to_string(),
        }]
    });
    ClaudeOutput {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PermissionRequest".to_string(),
            decision: ClaudeDecision {
                behavior: "allow".to_string(),
                message: None,
                updated_permissions,
            },
        },
    }
}

pub fn build_claude_denial(reason: String) -> ClaudeOutput {
    ClaudeOutput {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "PermissionRequest".to_string(),
            decision: ClaudeDecision {
                behavior: "deny".to_string(),
                message: Some(reason),
                updated_permissions: None,
            },
        },
    }
}

pub fn build_gemini_approval(feedback: Option<String>) -> GeminiOutput {
    GeminiOutput {
        system_message: feedback,
        decision: None,
        reason: None,
    }
}

pub fn build_gemini_denial(reason: String) -> GeminiOutput {
    GeminiOutput {
        system_message: None,
        decision: Some("deny".to_string()),
        reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_event() {
        let json_str = r#"{
            "permission_mode": "default",
            "tool_input": {
                "plan": "My implementation plan"
            }
        }"#;
        let parsed = parse_input(json_str).unwrap();
        assert_eq!(parsed.permission_mode, Some("default".to_string()));
        assert_eq!(
            parsed.tool_input.and_then(|t| t.plan),
            Some("My implementation plan".to_string())
        );
    }

    #[test]
    fn test_parse_gemini_event() {
        let json_str = r#"{
            "session_id": "session-123",
            "transcript_path": "/path/to/transcript.json",
            "tool_input": {
                "plan_filename": "plan-001.md"
            }
        }"#;
        let parsed = parse_input(json_str).unwrap();
        assert_eq!(parsed.session_id, Some("session-123".to_string()));
        assert_eq!(
            parsed.tool_input.and_then(|t| t.plan_filename),
            Some("plan-001.md".to_string())
        );
    }

    #[test]
    fn test_claude_approval() {
        let approval = build_claude_approval(Some("full".to_string()));
        assert_eq!(
            approval.hook_specific_output.hook_event_name,
            "PermissionRequest"
        );
        assert_eq!(approval.hook_specific_output.decision.behavior, "allow");
        let perms = approval
            .hook_specific_output
            .decision
            .updated_permissions
            .unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].r#type, "setMode");
        assert_eq!(perms[0].mode, "full");
    }

    #[test]
    fn test_claude_denial() {
        let denial = build_claude_denial("Plan was not detailed enough".to_string());
        assert_eq!(
            denial.hook_specific_output.hook_event_name,
            "PermissionRequest"
        );
        assert_eq!(denial.hook_specific_output.decision.behavior, "deny");
        assert_eq!(
            denial.hook_specific_output.decision.message,
            Some("Plan was not detailed enough".to_string())
        );
    }

    #[test]
    fn test_gemini_approval() {
        let approval = build_gemini_approval(Some("Success feedback".to_string()));
        assert_eq!(
            approval.system_message,
            Some("Success feedback".to_string())
        );
        assert_eq!(approval.decision, None);
    }

    #[test]
    fn test_gemini_denial() {
        let denial = build_gemini_denial("Needs more tests".to_string());
        assert_eq!(denial.decision, Some("deny".to_string()));
        assert_eq!(denial.reason, Some("Needs more tests".to_string()));
    }
}
