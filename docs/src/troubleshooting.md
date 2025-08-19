# Troubleshooting

## Unable to connect to S3 or compatible service

- First, check if you can connect using the AWS CLI with the same settings.
  - Are your AWS credentials configured properly?
    - This includes checking `~/.aws/credentials`, environment variables, or any credential provider chain used by the AWS CLI.
  - Are the necessary permissions set correctly?
    - This includes IAM policies, roles, and bucket policies that allow operations like `s3:ListBucket` or `s3:GetObject`.
  - If you're using an S3-compatible service:
    - Is the `endpoint-url` set correctly?
    - Are you using the appropriate `path-style` access setting?
- You may be able to find more details about the error by looking at the `$STU_ROOT_DIR/error.log`.

## Can't preview images

- Set `preview.image = true` in config.toml.
- Images are displayed using Sixel, [iTerm2 Inline Images Protocol](https://iterm2.com/documentation-images.html), and [kitty Terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/). This feature will not work if you are using a terminal emulator that does not support any of these protocols.
