use std::fs;
use std::path::PathBuf;
use futures_util::{SinkExt, StreamExt};
use supertools_cli::protocol;
use supertools_cli::server;

#[tokio::test]
async fn test_integration_claude_flow() {
    let input_json = r#"{
        "permission_mode": "default",
        "tool_input": {
            "plan": "Review this awesome Rust code plan"
        }
    }"#;

    // 1. Parse input
    let parsed_input = protocol::parse_input(input_json).expect("Failed to parse Claude event");
    let plan = parsed_input.tool_input.as_ref().and_then(|t| t.plan.as_ref()).expect("Plan missing");
    let perm_mode = parsed_input.permission_mode.clone();

    // 2. Start server
    let (server_handle, url) = server::start_server(0, plan.clone(), "plan".to_string())
        .await
        .expect("Failed to start server");

    let ws_url = url.replace("http://", "ws://") + "/ws";

    // 3. Connect client
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WS");

    // 4. Read review data
    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = msg.to_text().unwrap();
    let server_msg: server::ServerMessage = serde_json::from_str(text).unwrap();
    assert_eq!(
        server_msg,
        server::ServerMessage::ReviewData {
            content: "Review this awesome Rust code plan".to_string(),
            mode: "plan".to_string()
        }
    );

    // 5. Send decision (approved)
    let decision = server::ClientMessage::Decision {
        approved: true,
        feedback: None,
    };
    ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&decision).unwrap().into()))
        .await
        .unwrap();

    // 6. Await server output
    let decision_result = server_handle.await.expect("Server handle failed");
    assert_eq!(decision_result.approved, true);

    // 7. Format output
    let output = protocol::build_claude_approval(perm_mode);
    let output_json = serde_json::to_string(&output).unwrap();
    
    // Assert correct Claude structure
    assert!(output_json.contains("PermissionRequest"));
    assert!(output_json.contains("allow"));
    assert!(output_json.contains("setMode"));
}

#[tokio::test]
async fn test_integration_gemini_flow() {
    // Write a temp directory structure simulating Gemini CLI environment
    let temp_dir = std::env::temp_dir().join("supertools_test_gemini");
    let plans_dir = temp_dir.join("chats").join("session-abc").join("plans");
    fs::create_dir_all(&plans_dir).unwrap();

    let plan_file = plans_dir.join("plan.md");
    fs::write(&plan_file, "Gemini plan content").unwrap();

    let transcript_file = temp_dir.join("chats").join("session-abc.json");
    fs::write(&transcript_file, "{}").unwrap();

    let input_json = format!(
        r#"{{
            "session_id": "chats/session-abc",
            "transcript_path": {:?},
            "tool_input": {{
                "plan_filename": "plan.md"
            }}
        }}"#,
        transcript_file.to_string_lossy().replace("\\", "/")
    );

    // 1. Parse input
    let parsed_input = protocol::parse_input(&input_json).expect("Failed to parse Gemini event");
    
    // Resolve plan path: dirname(dirname(transcript_path)) / session_id / plans / plan_filename
    let transcript_path = PathBuf::from(parsed_input.transcript_path.as_ref().unwrap());
    let project_temp_dir = transcript_path.parent().unwrap().parent().unwrap();
    let plan_filepath = project_temp_dir
        .join(parsed_input.session_id.as_ref().unwrap())
        .join("plans")
        .join(parsed_input.tool_input.as_ref().unwrap().plan_filename.as_ref().unwrap());

    let plan_content = fs::read_to_string(plan_filepath).unwrap();
    assert_eq!(plan_content, "Gemini plan content");

    // 2. Start server
    let (server_handle, url) = server::start_server(0, plan_content, "plan".to_string())
        .await
        .expect("Failed to start server");

    let ws_url = url.replace("http://", "ws://") + "/ws";

    // 3. Connect client
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Failed to connect to WS");

    // 4. Read review data
    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = msg.to_text().unwrap();
    let server_msg: server::ServerMessage = serde_json::from_str(text).unwrap();
    assert_eq!(
        server_msg,
        server::ServerMessage::ReviewData {
            content: "Gemini plan content".to_string(),
            mode: "plan".to_string()
        }
    );

    // 5. Send decision (denied)
    let decision = server::ClientMessage::Decision {
        approved: false,
        feedback: Some("Needs more tests".to_string()),
    };
    ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&decision).unwrap().into()))
        .await
        .unwrap();

    // 6. Await server output
    let decision_result = server_handle.await.expect("Server handle failed");
    assert_eq!(decision_result.approved, false);
    assert_eq!(decision_result.feedback, Some("Needs more tests".to_string()));

    // 7. Format output
    let output = protocol::build_gemini_denial("Needs more tests".to_string());
    let output_json = serde_json::to_string(&output).unwrap();
    
    // Assert correct Gemini structure
    assert!(output_json.contains("deny"));
    assert!(output_json.contains("Needs more tests"));

    // Cleanup temp dir
    fs::remove_dir_all(&temp_dir).unwrap_or(());
}
