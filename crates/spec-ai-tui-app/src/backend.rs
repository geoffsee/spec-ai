use anyhow::Result;
use spec_ai_core::cli::{formatting, parse_command, CliState, Command};
use spec_ai_core::types::Message;
use std::path::PathBuf;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Requests sent from the UI to the backend worker.
#[derive(Debug)]
pub enum BackendRequest {
    Submit(String),
}

/// Events emitted by the backend worker to drive the UI.
#[derive(Debug)]
pub enum BackendEvent {
    Initialized {
        agent: Option<String>,
        messages: Vec<Message>,
        reasoning: Vec<String>,
        status: String,
    },
    CommandResult {
        response: Option<String>,
        new_messages: Vec<Message>,
        reasoning: Vec<String>,
        status: String,
    },
    Error {
        context: String,
        message: String,
    },
    Quit,
}

/// Handle containing the channels used by the TUI to talk to the backend worker.
pub struct BackendHandle {
    pub request_tx: UnboundedSender<BackendRequest>,
    pub event_rx: UnboundedReceiver<BackendEvent>,
}

/// Spawn the backend worker that owns CliState and performs all agent operations.
pub fn spawn_backend(config_path: Option<PathBuf>) -> Result<BackendHandle> {
    let (request_tx, mut request_rx) = unbounded_channel();
    let (event_tx, event_rx) = unbounded_channel();

    let config_path = config_path.clone();
    tokio::spawn(async move {
        if let Err(err) = run_backend_loop(&mut request_rx, &event_tx, config_path).await {
            let _ = event_tx.send(BackendEvent::Error {
                context: "startup".to_string(),
                message: err.to_string(),
            });
        }
    });

    Ok(BackendHandle {
        request_tx,
        event_rx,
    })
}

async fn run_backend_loop(
    request_rx: &mut UnboundedReceiver<BackendRequest>,
    event_tx: &UnboundedSender<BackendEvent>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    // Force plain text output so we can render cleanly in our own UI.
    formatting::set_plain_text_mode(true);

    let mut cli_state = initialize_cli_state(config_path)?;
    let _ = cli_state.agent.load_history(200);

    let agent_name = cli_state.registry.active_name();
    let initial_messages = cli_state.agent.conversation_history().to_vec();
    cli_state.status_message = "Status: awaiting input".to_string();

    let _ = event_tx.send(BackendEvent::Initialized {
        agent: agent_name,
        messages: initial_messages,
        reasoning: cli_state.reasoning_messages.clone(),
        status: cli_state.status_message.clone(),
    });

    while let Some(request) = request_rx.recv().await {
        match request {
            BackendRequest::Submit(input) => {
                let command = parse_command(&input);
                cli_state.status_message = status_message_for_command(&command);

                let start_len = cli_state.agent.conversation_history().len();
                match cli_state.handle_line(&input).await {
                    Ok(output) => {
                        if output.as_deref() == Some("__QUIT__") {
                            let _ = event_tx.send(BackendEvent::Quit);
                            break;
                        }

                        let history = cli_state.agent.conversation_history().to_vec();
                        let new_messages: Vec<Message> =
                            history.into_iter().skip(start_len).collect();

                        // Return to idle after handling the command
                        cli_state.status_message = "Status: awaiting input".to_string();

                        let _ = event_tx.send(BackendEvent::CommandResult {
                            response: output,
                            new_messages,
                            reasoning: cli_state.reasoning_messages.clone(),
                            status: cli_state.status_message.clone(),
                        });
                    }
                    Err(err) => {
                        cli_state.status_message = "Status: error".to_string();
                        let _ = event_tx.send(BackendEvent::Error {
                            context: input,
                            message: err.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn initialize_cli_state(config_path: Option<PathBuf>) -> Result<CliState> {
    // Prefer explicit path, then env override, then crate-local config.
    let chosen = config_path
        .or_else(|| std::env::var("SPEC_AI_TUI_CONFIG").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec-ai.config.toml"));

    CliState::initialize_with_path(Some(chosen))
}

fn status_message_for_command(command: &Command) -> String {
    match command {
        Command::Empty => "Status: awaiting input".to_string(),
        Command::Help => "Status: showing help".to_string(),
        Command::Quit => "Status: exiting".to_string(),
        Command::ConfigReload => "Status: reloading configuration".to_string(),
        Command::ConfigShow => "Status: displaying configuration".to_string(),
        Command::PolicyReload => "Status: reloading policies".to_string(),
        Command::SwitchAgent(name) => format!("Status: switching to agent '{}'", name),
        Command::ListAgents => "Status: listing agents".to_string(),
        Command::MemoryShow(Some(limit)) => {
            format!("Status: showing last {} messages", limit)
        }
        Command::MemoryShow(None) => "Status: showing recent messages".to_string(),
        Command::SessionNew(Some(id)) => format!("Status: starting session '{}'", id),
        Command::SessionNew(None) => "Status: starting new session".to_string(),
        Command::SessionList => "Status: listing sessions".to_string(),
        Command::SessionSwitch(id) => format!("Status: switching to session '{}'", id),
        Command::GraphEnable => "Status: showing graph enable instructions".to_string(),
        Command::GraphDisable => "Status: showing graph disable instructions".to_string(),
        Command::GraphStatus => "Status: showing graph status".to_string(),
        Command::GraphShow(Some(limit)) => {
            format!("Status: inspecting graph (limit {})", limit)
        }
        Command::GraphShow(None) => "Status: inspecting graph".to_string(),
        Command::GraphClear => "Status: clearing session graph".to_string(),
        Command::SyncList => "Status: listing sync-enabled graphs".to_string(),
        Command::Init(_) => "Status: bootstrapping repository graph".to_string(),
        Command::ListenStart(duration) => {
            let mut status = "Status: starting background transcription".to_string();
            if let Some(d) = duration {
                status.push_str(&format!(" for {} seconds", d));
            }
            status
        }
        Command::ListenStop => "Status: stopping transcription".to_string(),
        Command::ListenStatus => "Status: checking transcription status".to_string(),
        Command::Listen(scenario, duration) => {
            let mut status = "Status: starting audio transcription".to_string();
            if let Some(s) = scenario {
                status.push_str(&format!(" (scenario: {})", s));
            }
            if let Some(d) = duration {
                status.push_str(&format!(" for {} seconds", d));
            }
            status
        }
        Command::RunSpec(path) => format!("Status: executing spec '{}'", path.display()),
        Command::PasteStart => {
            "Status: entering paste mode (end with /end on its own line)".to_string()
        }
        Command::SpeechToggle(Some(true)) => "Status: enabling speech playback".to_string(),
        Command::SpeechToggle(Some(false)) => "Status: disabling speech playback".to_string(),
        Command::SpeechToggle(None) => "Status: toggling speech playback".to_string(),
        Command::Message(_) => "Status: running agent step".to_string(),
        Command::Refresh(_) => "Status: refreshing internal knowledge graph".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn status_message_empty_command() {
        let status = status_message_for_command(&Command::Empty);
        assert!(status.contains("awaiting input"));
    }

    #[test]
    fn status_message_help_command() {
        let status = status_message_for_command(&Command::Help);
        assert!(status.contains("help"));
    }

    #[test]
    fn status_message_quit_command() {
        let status = status_message_for_command(&Command::Quit);
        assert!(status.contains("exiting"));
    }

    #[test]
    fn status_message_config_reload() {
        let status = status_message_for_command(&Command::ConfigReload);
        assert!(status.contains("reloading configuration"));
    }

    #[test]
    fn status_message_config_show() {
        let status = status_message_for_command(&Command::ConfigShow);
        assert!(status.contains("displaying configuration"));
    }

    #[test]
    fn status_message_policy_reload() {
        let status = status_message_for_command(&Command::PolicyReload);
        assert!(status.contains("reloading policies"));
    }

    #[test]
    fn status_message_switch_agent() {
        let status = status_message_for_command(&Command::SwitchAgent("test-agent".to_string()));
        assert!(status.contains("switching"));
        assert!(status.contains("test-agent"));
    }

    #[test]
    fn status_message_list_agents() {
        let status = status_message_for_command(&Command::ListAgents);
        assert!(status.contains("listing agents"));
    }

    #[test]
    fn status_message_memory_show_with_limit() {
        let status = status_message_for_command(&Command::MemoryShow(Some(10)));
        assert!(status.contains("10"));
        assert!(status.contains("messages"));
    }

    #[test]
    fn status_message_memory_show_no_limit() {
        let status = status_message_for_command(&Command::MemoryShow(None));
        assert!(status.contains("recent messages"));
    }

    #[test]
    fn status_message_session_new_with_id() {
        let status =
            status_message_for_command(&Command::SessionNew(Some("my-session".to_string())));
        assert!(status.contains("my-session"));
    }

    #[test]
    fn status_message_session_new_no_id() {
        let status = status_message_for_command(&Command::SessionNew(None));
        assert!(status.contains("new session"));
    }

    #[test]
    fn status_message_session_list() {
        let status = status_message_for_command(&Command::SessionList);
        assert!(status.contains("listing sessions"));
    }

    #[test]
    fn status_message_session_switch() {
        let status = status_message_for_command(&Command::SessionSwitch("sess-1".to_string()));
        assert!(status.contains("switching"));
        assert!(status.contains("sess-1"));
    }

    #[test]
    fn status_message_graph_enable() {
        let status = status_message_for_command(&Command::GraphEnable);
        assert!(status.contains("graph enable"));
    }

    #[test]
    fn status_message_graph_disable() {
        let status = status_message_for_command(&Command::GraphDisable);
        assert!(status.contains("graph disable"));
    }

    #[test]
    fn status_message_graph_status() {
        let status = status_message_for_command(&Command::GraphStatus);
        assert!(status.contains("graph status"));
    }

    #[test]
    fn status_message_graph_show_with_limit() {
        let status = status_message_for_command(&Command::GraphShow(Some(50)));
        assert!(status.contains("inspecting graph"));
        assert!(status.contains("50"));
    }

    #[test]
    fn status_message_graph_show_no_limit() {
        let status = status_message_for_command(&Command::GraphShow(None));
        assert!(status.contains("inspecting graph"));
    }

    #[test]
    fn status_message_graph_clear() {
        let status = status_message_for_command(&Command::GraphClear);
        assert!(status.contains("clearing"));
    }

    #[test]
    fn status_message_sync_list() {
        let status = status_message_for_command(&Command::SyncList);
        assert!(status.contains("sync"));
    }

    #[test]
    fn status_message_init() {
        let status = status_message_for_command(&Command::Init(None));
        assert!(status.contains("bootstrapping"));
    }

    #[test]
    fn status_message_listen_start_no_duration() {
        let status = status_message_for_command(&Command::ListenStart(None));
        assert!(status.contains("transcription"));
    }

    #[test]
    fn status_message_listen_start_with_duration() {
        let status = status_message_for_command(&Command::ListenStart(Some(30)));
        assert!(status.contains("transcription"));
        assert!(status.contains("30"));
    }

    #[test]
    fn status_message_listen_stop() {
        let status = status_message_for_command(&Command::ListenStop);
        assert!(status.contains("stopping"));
    }

    #[test]
    fn status_message_listen_status() {
        let status = status_message_for_command(&Command::ListenStatus);
        assert!(status.contains("checking"));
    }

    #[test]
    fn status_message_listen_with_scenario() {
        let status =
            status_message_for_command(&Command::Listen(Some("meeting".to_string()), None));
        assert!(status.contains("meeting"));
    }

    #[test]
    fn status_message_listen_with_duration() {
        let status = status_message_for_command(&Command::Listen(None, Some(60)));
        assert!(status.contains("60"));
    }

    #[test]
    fn status_message_run_spec() {
        let status =
            status_message_for_command(&Command::RunSpec(PathBuf::from("specs/test.spec")));
        assert!(status.contains("executing"));
        assert!(status.contains("test.spec"));
    }

    #[test]
    fn status_message_paste_start() {
        let status = status_message_for_command(&Command::PasteStart);
        assert!(status.contains("paste mode"));
    }

    #[test]
    fn status_message_speech_toggle_enable() {
        let status = status_message_for_command(&Command::SpeechToggle(Some(true)));
        assert!(status.contains("enabling"));
    }

    #[test]
    fn status_message_speech_toggle_disable() {
        let status = status_message_for_command(&Command::SpeechToggle(Some(false)));
        assert!(status.contains("disabling"));
    }

    #[test]
    fn status_message_speech_toggle_none() {
        let status = status_message_for_command(&Command::SpeechToggle(None));
        assert!(status.contains("toggling"));
    }

    #[test]
    fn status_message_message_command() {
        let status = status_message_for_command(&Command::Message("hello".to_string()));
        assert!(status.contains("agent step"));
    }

    #[test]
    fn status_message_refresh() {
        let status = status_message_for_command(&Command::Refresh(None));
        assert!(status.contains("refreshing"));
    }

    #[test]
    fn backend_event_initialized_fields() {
        let event = BackendEvent::Initialized {
            agent: Some("test".to_string()),
            messages: vec![],
            reasoning: vec!["reasoning".to_string()],
            status: "ready".to_string(),
        };
        match event {
            BackendEvent::Initialized {
                agent,
                messages,
                reasoning,
                status,
            } => {
                assert_eq!(agent, Some("test".to_string()));
                assert!(messages.is_empty());
                assert_eq!(reasoning.len(), 1);
                assert_eq!(status, "ready");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn backend_event_error_fields() {
        let event = BackendEvent::Error {
            context: "ctx".to_string(),
            message: "msg".to_string(),
        };
        match event {
            BackendEvent::Error { context, message } => {
                assert_eq!(context, "ctx");
                assert_eq!(message, "msg");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn backend_request_submit_contains_text() {
        let request = BackendRequest::Submit("test input".to_string());
        match request {
            BackendRequest::Submit(text) => {
                assert_eq!(text, "test input");
            }
        }
    }
}
