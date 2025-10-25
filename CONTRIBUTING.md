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
- Creating a pull request does not necessarily require an issue. But if the problem is complex, creating an issue beforehand might make the process smoother.
- Do not include fixes that are not directly related to the pull request topic.

### Continuous Integration

We use [GitHub Actions](https://github.com/lusingander/stu/blob/master/.github/workflows/build.yml) to perform basic checks:

- Run both stable and MSRV versions of Rust.
- Run build, test, format, and lint.

## Development

- The `Makefile` and `tool` directories (go projects) are not relevant for development, so you don't need to worry about them.
  - These are tools for creating screenshots, etc.
  - It's ok if you can run `cargo build` and `cargo test` as a normal Rust project.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.

## Additional Information

If you have any questions or concerns, please use the [Discussions](https://github.com/lusingander/stu/discussions).
