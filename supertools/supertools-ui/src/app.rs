use crate::components::plan_review::PlanReview;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::Message;
use gloo_net::websocket::futures::WebSocket;
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    ReviewData { content: String, mode: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    Decision {
        approved: bool,
        feedback: Option<String>,
    },
}

#[component]
pub fn App() -> impl IntoView {
    let (content, set_content) = create_signal(String::new());
    let (mode, set_mode) = create_signal(String::new());
    let (feedback, set_feedback) = create_signal(String::new());
    let (session_closed, set_session_closed) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);

    // WS connection sender resource/signal
    let (ws_sender, set_ws_sender) = create_signal(None);

    // Initialize WebSocket connection
    create_effect(move |_| {
        let window = web_sys::window().expect("no global window exists");
        let location = window.location();
        let protocol = location.protocol().expect("failed to get protocol");
        let host = location.host().expect("failed to get host");

        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let ws_url = format!("{}//{}/ws", ws_protocol, host);

        let ws = match WebSocket::open(&ws_url) {
            Ok(ws) => ws,
            Err(e) => {
                set_error_msg.set(Some(format!("Failed to open WebSocket: {:?}", e)));
                return;
            }
        };

        let (mut write, mut read) = ws.split();
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<Message>();

        // Store the sender in our state (which is Clone!)
        set_ws_sender.set(Some(tx));

        // Spawn a future to write messages from the channel to the WebSocket
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                let _ = write.send(msg).await;
            }
        });

        // Spawn a future to read messages from the WebSocket
        spawn_local(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(ServerMessage::ReviewData { content, mode }) =
                            serde_json::from_str::<ServerMessage>(&text)
                        {
                            set_content.set(content);
                            set_mode.set(mode);
                        }
                    }
                    Ok(Message::Bytes(_)) => {}
                    Err(e) => {
                        set_error_msg.set(Some(format!("WebSocket error: {:?}", e)));
                        set_session_closed.set(true);
                        break;
                    }
                }
            }
            // If the socket closes, mark the session as closed
            set_session_closed.set(true);
        });
    });

    // Helper to send decision to CLI
    let send_decision = move |approved: bool| {
        if let Some(writer) = ws_sender.get_untracked() {
            let current_feedback = feedback.get_untracked();
            let feedback_opt = if current_feedback.trim().is_empty() {
                None
            } else {
                Some(current_feedback)
            };

            let decision = ClientMessage::Decision {
                approved,
                feedback: feedback_opt,
            };

            if let Ok(json) = serde_json::to_string(&decision) {
                let _ = writer.unbounded_send(Message::Text(json));
            }
            set_session_closed.set(true);
        }
    };

    view! {
        <div class="container">
            <header class="glass pane-content" style="padding: 16px 24px; margin-bottom: 24px;">
                <div class="logo">"Supertools Review Portal"</div>
                <div>
                    <span class="badge">
                        {move || {
                            let m = mode.get();
                            if m.is_empty() { "Connecting...".to_string() } else { m }
                        }}
                    </span>
                </div>
            </header>

            {move || {
                if let Some(err) = error_msg.get() {
                    view! {
                        <div class="glass pane-content" style="border-color: var(--danger); background: rgba(239, 68, 68, 0.05); text-align: center; padding: 40px;">
                            <div style="font-size: 48px; color: var(--danger); margin-bottom: 16px;">"⚠️"</div>
                            <h2 style="margin-bottom: 8px;">"Connection Error"</h2>
                            <p style="color: var(--text-secondary);">{err}</p>
                        </div>
                    }.into_view()
                } else if session_closed.get() {
                    view! {
                        <div class="glass session-closed-view">
                            <div class="session-closed-icon">"✓"</div>
                            <h2 class="session-closed-title">"Review Session Complete"</h2>
                            <p class="session-closed-text">
                                "The decision has been returned to your terminal session. You can now close this browser tab."
                            </p>
                        </div>
                    }.into_view()
                } else if mode.get() == "review" || mode.get() == "code" {
                    let diffs_parsed = serde_json::from_str::<Vec<crate::diff_parser::FileDiff>>(&content.get()).unwrap_or_default();
                    view! {
                        <crate::components::code_review::CodeReview
                            diffs=diffs_parsed
                            feedback=feedback.into()
                            set_feedback=set_feedback
                            on_submit=Callback::new(move |approved| send_decision(approved))
                        />
                    }.into_view()
                } else {
                    view! {
                        <div class="review-workspace">
                            // Left Pane: Content to Review
                            <div class="pane glass glow-hover">
                                <div class="pane-title" style="padding: 16px 20px 0 20px; border-bottom: 1px solid var(--border-color); padding-bottom: 12px; margin-bottom: 0;">
                                    "Plan Details"
                                </div>
                                <div class="pane-content">
                                    <PlanReview markdown=content.into() />
                                </div>
                            </div>

                            // Right Pane: Feedback & Actions
                            <div class="pane glass glow-hover" style="max-height: 450px;">
                                <div class="pane-title" style="padding: 16px 20px 0 20px; border-bottom: 1px solid var(--border-color); padding-bottom: 12px; margin-bottom: 0;">
                                    "Review Action"
                                </div>
                                <div class="pane-content controls-card">
                                    <p class="text-secondary" style="font-size: 14px; margin-bottom: 8px;">
                                        "Add annotations or request changes below, or click Approve to proceed immediately."
                                    </p>

                                    <textarea
                                        class="text-area"
                                        placeholder="Add change requests or feedback notes here..."
                                        prop:value=feedback
                                        on:input=move |ev| {
                                            set_feedback.set(event_target_value(&ev));
                                        }
                                    />

                                    <button
                                        class="button button-success"
                                        on:click=move |_| send_decision(true)
                                    >
                                        "Approve Plan"
                                    </button>

                                    <button
                                        class="button button-danger"
                                        on:click=move |_| send_decision(false)
                                    >
                                        "Request Changes"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}
