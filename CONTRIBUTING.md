# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution.

To ensure that your contributions are considered, please follow this guidelines. Contributions that do not adhere to these guidelines may not be accepted.

## Reporting Issues

Before reporting, please check if an issue with the same content already exists.

Also, please refer to [FAQ](https://lusingander.github.io/stu/faq/index.html).

### Reporting Bugs

When reporting a bug, please include the following information:

- Application version
- Version of the terminal emulator and the OS it's running on
- The service you are trying to connect to (if it is an S3 compatible storage)
- Information to reproduce the issue

### Suggesting Features

Suggestions are welcome, but any suggestions that do not follow the project's policies are unlikely to be accepted.

- Update functions (put, delete, etc.) are a low priority.
- There are no plans to support tools that are incompatible with S3.

## Pull Requests

We welcome pull requests, but please note that they are not guaranteed to be accepted. Following this guideline will increase the likelihood of your pull request being approved.

### Creating pull requests

- When creating a pull request, please ensure you follow the same guidelines as [mentioned for issues](#reporting-issues).
- An issue is not required for every pull request. For small or straightforward changes (such as documentation fixes or obvious bug fixes), feel free to open a pull request directly.
- For more complex changes or behavior-altering fixes, opening an issue first is strongly recommended to discuss the approach and avoid unnecessary rework.
- Do not include fixes that are not directly related to the pull request topic.

### Continuous Integration

We use [GitHub Actions](https://github.com/lusingander/stu/blob/master/.github/workflows/build.yml) to perform basic checks:

- Run build, unit tests, format, and lint with stable Rust and MSRV.
- Run these checks on Linux, macOS, and Windows, with MSRV checked on Linux only.
- Run terminal E2E tests in a separate Linux job with stable Rust and Docker.

## Development

- The `Makefile` and `tool` directories (go projects) are not relevant for development, so you don't need to worry about them.
  - These are tools for creating screenshots, etc.

Build with `cargo build` and run all tests with `cargo test`.

Terminal E2E tests require a running Docker daemon supporting Engine API 1.53 or later (Docker 29.2+). Testcontainers starts a dedicated LocalStack `3.2.0` container for each scenario and removes it afterwards. On Unix, containers are also removed when tests are interrupted with Ctrl+C. This is the same pinned image used for screenshot generation. The tests use dummy AWS credentials and isolated configuration directories; no AWS account or LocalStack auth token is required. If your Docker daemon uses a non-default socket, set `DOCKER_HOST` to its endpoint.

Run all terminal scenarios, or select one by its test name:

```sh
cargo test --test terminal_e2e
cargo test --test terminal_e2e navigates_into_nested_directories
```

Run only unit tests with `cargo test --bin stu`.

The scenarios in `tests/terminal_e2e.rs` operate the real binary through `termlens` and verify rendered terminal output. Shared container, fixture, and application setup lives in `tests/support`. Wait for the display changed by each operation instead of using fixed sleeps. Terminal wait failures include the screen, and unsuccessful scenarios also print the last lines of the application logs.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.

## Additional Information

If you have any questions or concerns, please use the [Discussions](https://github.com/lusingander/stu/discussions).
