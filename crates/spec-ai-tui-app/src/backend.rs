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
