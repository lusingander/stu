mod support;

use std::fs;

use anyhow::Result;
use rstest::rstest;
use termlens::Key;

use support::{at_path, footer_contains, selected_item, Storage, TestApp};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refreshes_buckets_and_opens_a_newly_created_bucket() -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-charlie"]).await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "") && selected_item(screen, "stu-alpha") && screen.contains("stu-charlie")
    })?;

    storage.create_bucket("stu-bravo").await?;
    storage
        .put_object("stu-bravo", "added.txt", b"new bucket")
        .await?;
    app.terminal.send(Key::Char('R'))?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "")
            && selected_item(screen, "stu-alpha")
            && screen.contains("stu-bravo")
            && screen.contains("stu-charlie")
    })?;

    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "stu-bravo"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_path(screen, "stu-bravo") && selected_item(screen, "added.txt"))?;

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filters_buckets_and_restores_the_full_list_after_returning() -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-beta", "stu-gamma"]).await?;
    storage
        .put_object("stu-beta", "chosen.txt", b"chosen bucket")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "stu-alpha") && screen.contains("stu-gamma"))?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Filter") && footer_contains(screen, "Close"))?;
    app.terminal.send_str("beta")?;
    app.terminal.send(Key::Enter)?;
    let filtered = app.terminal.snapshot_after(|screen| {
        footer_contains(screen, "Clear filter") && selected_item(screen, "stu-beta")
    })?;
    assert!(!filtered.contains("stu-alpha"), "{filtered}");
    assert!(!filtered.contains("stu-gamma"), "{filtered}");

    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_path(screen, "stu-beta") && screen.contains("chosen.txt"))?;

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        footer_contains(screen, "Clear filter") && selected_item(screen, "stu-beta")
    })?;
    app.terminal.send(Key::Esc)?;
    app.terminal.wait_until(|screen| {
        screen.contains("stu-alpha")
            && screen.contains("stu-beta")
            && screen.contains("stu-gamma")
            && !footer_contains(screen, "Clear filter")
    })?;

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sorts_and_filters_objects_before_opening_the_selected_details() -> Result<()> {
    let storage = Storage::new(&["stu-objects"]).await?;
    for (key, body) in [
        ("alpha.txt", "a"),
        ("bravo.log", "log"),
        ("charlie.txt", "charlie"),
    ] {
        storage
            .put_object("stu-objects", key, body.as_bytes())
            .await?;
    }
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-objects") && selected_item(screen, "alpha.txt")
    })?;

    app.terminal.send(Key::Char('o'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Last Modified (Asc)"))?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        selected_item(screen, "charlie.txt") && !screen.contains("Last Modified (Asc)")
    })?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Filter") && footer_contains(screen, "Close"))?;
    app.terminal.send_str(".txt")?;
    app.terminal.send(Key::Enter)?;
    let filtered = app.terminal.snapshot_after(|screen| {
        footer_contains(screen, "Clear filter")
            && selected_item(screen, "charlie.txt")
            && screen.contains("alpha.txt")
    })?;
    assert!(!filtered.contains("bravo.log"), "{filtered}");

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        screen.contains("Content-Type:")
            && screen.contains("text/plain")
            && screen.contains("7 B")
            && !screen.contains("Loading...")
    })?;

    app.terminal.send(Key::Backspace)?;
    let restored = app.terminal.snapshot_after(|screen| {
        footer_contains(screen, "Clear filter") && selected_item(screen, "charlie.txt")
    })?;
    assert!(!restored.contains("bravo.log"), "{restored}");
    assert!(!restored.contains("Content-Type:"), "{restored}");

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn previews_text_and_returns_to_the_selected_object() -> Result<()> {
    let storage = Storage::new(&["stu-preview"]).await?;
    storage
        .put_object("stu-preview", "alpha.txt", b"another object")
        .await?;
    let content = (1..=60)
        .map(|line| format!("preview line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    storage
        .put_object("stu-preview", "notes.txt", content.as_bytes())
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-preview") && selected_item(screen, "alpha.txt")
    })?;

    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "notes.txt"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        screen.contains("Content-Type:")
            && screen.contains("text/plain")
            && !screen.contains("Loading...")
    })?;

    app.terminal.send(Key::Char('p'))?;
    app.terminal.wait_until(|screen| {
        screen.contains("Preview [notes.txt]") && screen.contains("preview line 01")
    })?;
    app.terminal.send(Key::Char('G'))?;
    let bottom = app
        .terminal
        .snapshot_after(|screen| screen.contains("preview line 60"))?;
    assert!(!bottom.contains("preview line 01"), "{bottom}");

    app.terminal.send(Key::Char('g'))?;
    let top = app
        .terminal
        .snapshot_after(|screen| screen.contains("preview line 01"))?;
    assert!(!top.contains("preview line 60"), "{top}");

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        screen.contains("Content-Type:") && !screen.contains("Preview [notes.txt]")
    })?;
    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-preview")
            && selected_item(screen, "notes.txt")
            && !screen.contains("Content-Type:")
    })?;

    app.quit()
}

#[rstest]
#[case::from_list(false, None)]
#[case::from_details(true, None)]
#[case::save_as(true, Some("renamed.bin"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn downloads_an_object_to_the_expected_file(
    #[case] open_details: bool,
    #[case] save_as: Option<&str>,
) -> Result<()> {
    let storage = Storage::new(&["stu-download"]).await?;
    let content = b"download fixture\0\xff";
    storage
        .put_object("stu-download", "payload.bin", content)
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-download") && selected_item(screen, "payload.bin")
    })?;

    if open_details {
        app.terminal.send(Key::Enter)?;
        app.terminal.wait_until(|screen| {
            screen.contains("Content-Type:")
                && screen.contains("text/plain")
                && !screen.contains("Loading...")
        })?;
    }

    if let Some(name) = save_as {
        app.terminal.send(Key::Char('S'))?;
        app.terminal
            .wait_until(|screen| screen.contains("Save As"))?;
        app.terminal.send(Key::Ctrl('u'))?;
        app.terminal.send_str(name)?;
        app.terminal
            .wait_until(|screen| screen.contains("Save As") && screen.contains(name))?;
        app.terminal.send(Key::Enter)?;
    } else {
        app.terminal.send(Key::Char('s'))?;
    }
    app.terminal.wait_until(|screen| {
        footer_contains(screen, "Download completed successfully:")
            && !screen.contains("Loading...")
    })?;

    let directory = app.download_dir();
    let name = save_as.unwrap_or("payload.bin");
    assert_eq!(fs::read(directory.join(name))?, content);
    assert_eq!(fs::read_dir(&directory)?.count(), 1);

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn downloads_a_directory_with_its_relative_paths_and_contents() -> Result<()> {
    let storage = Storage::new(&["stu-download"]).await?;
    let intro = b"documentation";
    let config = br#"{"enabled":true}"#;
    storage
        .put_object("stu-download", "docs/intro.txt", intro)
        .await?;
    storage
        .put_object("stu-download", "docs/nested/config.json", config)
        .await?;
    storage
        .put_object("stu-download", "outside.txt", b"excluded")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-download")
            && selected_item(screen, "docs/")
            && screen.contains("outside.txt")
    })?;

    app.terminal.send(Key::Char('s'))?;
    app.terminal.wait_until(|screen| {
        screen.contains("2 objects (Total size:")
            && screen.contains("Do you want to proceed?")
            && !screen.contains("Loading...")
    })?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        footer_contains(screen, "Download completed successfully:")
            && !screen.contains("Loading...")
    })?;

    let directory = app.download_dir();
    assert_eq!(fs::read(directory.join("docs/intro.txt"))?, intro);
    assert_eq!(fs::read(directory.join("docs/nested/config.json"))?, config);
    assert_eq!(fs::read_dir(&directory)?.count(), 1);
    assert_eq!(fs::read_dir(directory.join("docs"))?.count(), 2);
    assert_eq!(fs::read_dir(directory.join("docs/nested"))?.count(), 1);

    app.quit()
}
