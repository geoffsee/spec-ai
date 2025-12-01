#[tokio::main]
async fn main() -> anyhow::Result<()> {
    spec_ai_tui_app::run_tui(None).await
}
