mod support;

use anyhow::Result;
use rstest::rstest;
use termlens::Key;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn navigates_into_nested_directories_and_back_to_the_bucket_list() -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-beta"]).await?;
    for key in ["root.txt", "docs/readme.txt", "docs/nested/deep.txt"] {
        storage.put_object("stu-alpha", key, b"fixture\n").await?;
    }
    storage
        .put_object("stu-beta", "other.txt", b"other bucket\n")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;

    app.terminal.wait_until(|screen| {
        screen.contains("stu-alpha")
            && screen.contains("stu-beta")
            && !screen.contains("Loading...")
    })?;

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-alpha") && screen.contains("docs/") && screen.contains("root.txt")
    })?;

    app.terminal.send(Key::Enter)?;
    let directory = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-alpha / docs")
            && screen.contains("nested/")
            && screen.contains("readme.txt")
    })?;
    assert!(!directory.contains("root.txt"), "{directory}");

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-alpha / docs / nested") && screen.contains("deep.txt")
    })?;

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-alpha / docs") && screen.contains("readme.txt")
    })?;

    app.terminal.send(Key::Backspace)?;
    app.terminal
        .wait_until(|screen| at_path(screen, "stu-alpha") && screen.contains("root.txt"))?;

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        screen.contains("stu-alpha")
            && screen.contains("stu-beta")
            && !screen.contains("Loading...")
    })?;

    app.terminal.send(Key::Char('j'))?;
    app.terminal.send(Key::Enter)?;
    let other_bucket = app
        .terminal
        .snapshot_after(|screen| at_path(screen, "stu-beta") && screen.contains("other.txt"))?;
    assert!(!other_bucket.contains("root.txt"), "{other_bucket}");

    app.quit()
}

#[rstest]
#[case::bucket(
    &["--bucket", "stu-alpha"],
    "stu-alpha",
    "root.txt",
    "readme.txt"
)]
#[case::prefix(
    &["--bucket", "stu-alpha", "--prefix", "docs/"],
    "stu-alpha / docs",
    "readme.txt",
    "root.txt"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn starts_at_the_requested_bucket_or_prefix(
    #[case] args: &[&str],
    #[case] path: &str,
    #[case] visible: &str,
    #[case] hidden: &str,
) -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-beta"]).await?;
    for key in ["root.txt", "docs/readme.txt"] {
        storage.put_object("stu-alpha", key, b"fixture\n").await?;
    }
    let mut app = TestApp::spawn(&storage.endpoint, args)?;

    let screen = app
        .terminal
        .snapshot_after(|screen| at_path(screen, path) && screen.contains(visible))?;
    assert!(!screen.contains(hidden), "{screen}");
    assert!(!screen.contains("stu-beta"), "{screen}");

    app.quit()
}
