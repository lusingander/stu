mod support;

use anyhow::Result;

use support::{at_path, Storage, TestApp};

// Keep runtime workers available while termlens waits synchronously for the PTY.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn starts_with_a_bucket_list_and_restores_the_terminal_on_quit() -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-beta"]).await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;

    app.terminal.wait_until(|screen| {
        screen.contains("stu-alpha")
            && screen.contains("stu-beta")
            && !screen.contains("Loading...")
    })?;
    assert!(app.terminal.screen().alternate_screen());

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opens_the_only_bucket_automatically() -> Result<()> {
    let storage = Storage::new(&["stu-only"]).await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;

    app.terminal
        .wait_until(|screen| at_path(screen, "stu-only"))?;

    app.quit()
}
