mod support;

use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use rstest::rstest;
use termlens::Key;

use support::{at_path, footer_contains, list_rows, selected_item, Storage, TestApp};

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
async fn returns_from_an_empty_bucket_and_opens_an_object_in_another_bucket() -> Result<()> {
    let storage = Storage::new(&["stu-empty", "stu-populated"]).await?;
    storage
        .put_object("stu-populated", "available.txt", b"available")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "")
            && selected_item(screen, "stu-empty")
            && screen.contains("stu-populated")
    })?;

    app.terminal.send(Key::Enter)?;
    let empty = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-empty") && footer_contains(screen, "Go back")
    })?;
    assert!(list_rows(&empty).is_empty(), "{empty}");

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "")
            && selected_item(screen, "stu-empty")
            && screen.contains("stu-populated")
    })?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "stu-populated"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-populated") && selected_item(screen, "available.txt")
    })?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-populated")
            && screen.contains("Content-Type:")
            && screen.contains("9 B")
    })?;

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
async fn refreshes_objects_and_preserves_the_path_filter_sort_and_selection() -> Result<()> {
    let storage = Storage::new(&["stu-refresh"]).await?;
    for (key, body) in [
        ("outside.txt", "outside"),
        ("docs/alpha.txt", "a"),
        ("docs/bravo.log", "log"),
        ("docs/charlie.txt", "charlie"),
    ] {
        storage
            .put_object("stu-refresh", key, body.as_bytes())
            .await?;
    }
    let mut app = TestApp::spawn(
        &storage.endpoint,
        &["--bucket", "stu-refresh", "--prefix", "docs/"],
    )?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-refresh / docs") && selected_item(screen, "alpha.txt")
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
    app.terminal.wait_until(|screen| {
        footer_contains(screen, "Clear filter") && selected_item(screen, "charlie.txt")
    })?;

    // Insert ahead of the selection so retaining its index would select the wrong file.
    storage
        .put_object("stu-refresh", "docs/delta.txt", b"new delta")
        .await?;
    app.terminal.send(Key::Char('R'))?;
    let refreshed = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-refresh / docs")
            && footer_contains(screen, "Clear filter")
            && selected_item(screen, "charlie.txt")
            && screen.contains("delta.txt")
    })?;
    let rows = list_rows(&refreshed);
    assert_eq!(rows.len(), 3, "{refreshed}");
    for (row, name) in rows.iter().zip(["delta.txt", "charlie.txt", "alpha.txt"]) {
        assert!(row.starts_with(name), "{refreshed}");
    }
    assert!(!refreshed.contains("bravo.log"), "{refreshed}");
    assert!(!refreshed.contains("outside.txt"), "{refreshed}");

    app.terminal.send(Key::Char('k'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "delta.txt"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-refresh / docs")
            && screen.contains("Content-Type:")
            && screen.contains("text/plain")
            && screen.contains("9 B")
    })?;

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
async fn clears_a_bucket_filter_with_no_matches_and_opens_a_bucket() -> Result<()> {
    let storage = Storage::new(&["stu-alpha", "stu-beta"]).await?;
    storage
        .put_object("stu-beta", "chosen.txt", b"chosen bucket")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "") && selected_item(screen, "stu-alpha") && screen.contains("stu-beta")
    })?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Filter") && footer_contains(screen, "Close"))?;
    app.terminal.send_str("no-match")?;
    app.terminal.send(Key::Enter)?;
    let filtered = app
        .terminal
        .snapshot_after(|screen| at_path(screen, "") && footer_contains(screen, "Clear filter"))?;
    assert!(list_rows(&filtered).is_empty(), "{filtered}");

    app.terminal.send(Key::Esc)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "")
            && !footer_contains(screen, "Clear filter")
            && selected_item(screen, "stu-alpha")
            && screen.contains("stu-beta")
    })?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "stu-beta"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_path(screen, "stu-beta") && selected_item(screen, "chosen.txt"))?;

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
async fn clears_an_object_filter_with_no_matches_and_opens_an_object() -> Result<()> {
    let storage = Storage::new(&["stu-filter"]).await?;
    storage.put_object("stu-filter", "alpha.txt", b"a").await?;
    storage
        .put_object("stu-filter", "bravo.log", b"bravo")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-filter")
            && selected_item(screen, "alpha.txt")
            && screen.contains("bravo.log")
    })?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Filter") && footer_contains(screen, "Close"))?;
    app.terminal.send_str("no-match")?;
    app.terminal.send(Key::Enter)?;
    let filtered = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-filter") && footer_contains(screen, "Clear filter")
    })?;
    assert!(list_rows(&filtered).is_empty(), "{filtered}");

    app.terminal.send(Key::Esc)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-filter")
            && !footer_contains(screen, "Clear filter")
            && selected_item(screen, "alpha.txt")
            && screen.contains("bravo.log")
    })?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "bravo.log"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-filter")
            && screen.contains("Content-Type:")
            && screen.contains("text/plain")
            && screen.contains("5 B")
    })?;

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pages_to_an_offscreen_object_and_restores_the_list_after_details() -> Result<()> {
    let storage = Storage::new(&["stu-scroll"]).await?;
    for number in 1..=40 {
        let key = format!("file{number:02}.txt");
        let content = vec![b'x'; number];
        storage.put_object("stu-scroll", &key, &content).await?;
    }
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    let first_page = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-scroll") && selected_item(screen, "file01.txt")
    })?;
    let target_number = list_rows(&first_page).len() + 1;
    let target = format!("file{target_number:02}.txt");
    assert!(target_number <= 40, "{first_page}");
    assert!(!first_page.contains(&target), "{first_page}");

    app.terminal.send(Key::Ctrl('f'))?;
    let next_page = app
        .terminal
        .snapshot_after(|screen| at_path(screen, "stu-scroll") && selected_item(screen, &target))?;
    assert!(!next_page.contains("file01.txt"), "{next_page}");

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-scroll")
            && screen.contains("Content-Type:")
            && screen.contains("text/plain")
            && screen.contains(&format!("{target_number} B"))
    })?;

    app.terminal.send(Key::Backspace)?;
    let restored = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-scroll")
            && selected_item(screen, &target)
            && !screen.contains("Content-Type:")
    })?;
    assert_eq!(list_rows(&restored), list_rows(&next_page), "{restored}");

    app.terminal.send(Key::Ctrl('b'))?;
    let previous_page = app.terminal.snapshot_after(|screen| {
        at_path(screen, "stu-scroll") && selected_item(screen, "file01.txt")
    })?;
    assert_eq!(
        list_rows(&previous_page),
        list_rows(&first_page),
        "{previous_page}"
    );

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

    let name = save_as.unwrap_or("payload.bin");
    assert_eq!(
        app.downloaded_files()?,
        BTreeMap::from([(PathBuf::from(name), content.to_vec())])
    );

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

    assert_eq!(
        app.downloaded_files()?,
        BTreeMap::from([
            (PathBuf::from("docs/intro.txt"), intro.to_vec()),
            (PathBuf::from("docs/nested/config.json"), config.to_vec()),
        ])
    );

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn downloads_an_entire_bucket_with_its_relative_paths_and_contents() -> Result<()> {
    let storage = Storage::new(&["stu-bucket", "stu-other"]).await?;
    let objects: [(&str, &[u8]); 3] = [
        ("root.txt", b"bucket root"),
        ("docs/intro.txt", b"introduction"),
        ("docs/nested/payload.bin", b"nested\0\xff"),
    ];
    for (key, content) in objects {
        storage.put_object("stu-bucket", key, content).await?;
    }
    for key in ["root.txt", "docs/nested/payload.bin", "other-only.txt"] {
        storage
            .put_object("stu-other", key, b"excluded bucket")
            .await?;
    }
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "") && selected_item(screen, "stu-bucket") && screen.contains("stu-other")
    })?;

    app.terminal.send(Key::Char('s'))?;
    app.terminal.wait_until(|screen| {
        screen.contains("3 objects (Total size:")
            && screen.contains("Do you want to proceed?")
            && !screen.contains("Loading...")
    })?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "") && footer_contains(screen, "Download completed successfully:")
    })?;

    let expected: BTreeMap<_, _> = objects
        .into_iter()
        .map(|(key, content)| (PathBuf::from("stu-bucket").join(key), content.to_vec()))
        .collect();
    assert_eq!(app.downloaded_files()?, expected);

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancels_a_directory_download_and_can_download_it_afterwards() -> Result<()> {
    let storage = Storage::new(&["stu-cancel"]).await?;
    let objects: [(&str, &[u8]); 2] = [
        ("docs/intro.txt", b"documentation"),
        ("docs/nested/payload.bin", b"nested\0\xff"),
    ];
    for (key, content) in objects {
        storage.put_object("stu-cancel", key, content).await?;
    }
    storage
        .put_object("stu-cancel", "outside.txt", b"excluded")
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-cancel")
            && selected_item(screen, "docs/")
            && screen.contains("outside.txt")
    })?;

    app.terminal.send(Key::Char('s'))?;
    app.terminal.wait_until(|screen| {
        screen.contains("2 objects (Total size:")
            && screen.contains("Do you want to proceed?")
            && !screen.contains("Loading...")
    })?;
    app.terminal.send(Key::Char('l'))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-cancel")
            && selected_item(screen, "docs/")
            && footer_contains(screen, "Open")
            && !screen.contains("Do you want to proceed?")
    })?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-cancel") && selected_item(screen, "outside.txt")
    })?;
    assert!(app.downloaded_files()?.is_empty());

    app.terminal.send(Key::Char('k'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "docs/"))?;
    app.terminal.send(Key::Char('s'))?;
    app.terminal.wait_until(|screen| {
        screen.contains("2 objects (Total size:")
            && screen.contains("Do you want to proceed?")
            && !screen.contains("Loading...")
    })?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-cancel") && footer_contains(screen, "Download completed successfully:")
    })?;

    let expected: BTreeMap<_, _> = objects
        .into_iter()
        .map(|(key, content)| (PathBuf::from(key), content.to_vec()))
        .collect();
    assert_eq!(app.downloaded_files()?, expected);

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancels_save_as_and_downloads_the_selected_object_with_its_original_name() -> Result<()> {
    let storage = Storage::new(&["stu-save-cancel"]).await?;
    let content = b"download after cancellation\0\xff";
    storage
        .put_object("stu-save-cancel", "alpha.txt", b"another object")
        .await?;
    storage
        .put_object("stu-save-cancel", "payload.bin", content)
        .await?;
    let mut app = TestApp::spawn(&storage.endpoint, &[])?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel")
            && selected_item(screen, "alpha.txt")
            && screen.contains("payload.bin")
    })?;
    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "payload.bin"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel") && screen.contains("Content-Type:")
    })?;

    app.terminal.send(Key::Char('S'))?;
    app.terminal
        .wait_until(|screen| screen.contains("Save As") && footer_contains(screen, "Close"))?;
    app.terminal.send(Key::Ctrl('u'))?;
    app.terminal.send_str("cancelled.bin")?;
    app.terminal
        .wait_until(|screen| screen.contains("Save As") && screen.contains("cancelled.bin"))?;
    app.terminal.send(Key::Esc)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel")
            && screen.contains("Content-Type:")
            && footer_contains(screen, "Preview")
            && !screen.contains("Save As")
    })?;
    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel")
            && selected_item(screen, "payload.bin")
            && !screen.contains("Content-Type:")
    })?;
    assert!(app.downloaded_files()?.is_empty());

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel") && screen.contains("Content-Type:")
    })?;
    app.terminal.send(Key::Char('s'))?;
    app.terminal.wait_until(|screen| {
        at_path(screen, "stu-save-cancel")
            && footer_contains(screen, "Download completed successfully:")
    })?;
    assert_eq!(
        app.downloaded_files()?,
        BTreeMap::from([(PathBuf::from("payload.bin"), content.to_vec())])
    );

    app.quit()
}
