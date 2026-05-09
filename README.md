# yoink

`yoink` is a command-line tool that walks a directory tree, reads text files, and concatenates them into a single document suitable for pasting into LLM chats.

## Requirements

- [Rust](https://www.rust-lang.org/) toolchain, only if you [build from source](#install).

## Releases

Prebuilt binaries for Linux, Windows, and macOS are published on [GitHub Releases](https://github.com/andrew264/yoink/releases). Pick the archive that matches your OS and CPU architecture. Each binary upload is accompanied by a `.sha256` checksum file.


| OS      | Archives                                                                          |
| ------- | --------------------------------------------------------------------------------- |
| Linux   | `yoink-x86_64-unknown-linux-gnu.tar.gz`, `yoink-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | `yoink-x86_64-pc-windows-msvc.zip`, `yoink-aarch64-pc-windows-msvc.zip`           |
| macOS   | `yoink-x86_64-apple-darwin.tar.gz`, `yoink-aarch64-apple-darwin.tar.gz`           |


Extract the archive and put `yoink` on your `PATH`. On Windows, use `yoink.exe` from the zip.

## Install

From a clone of this repository:

```sh
cargo install --path .
```

## Usage

With no subcommand, `yoink` processes the given path (default: current directory), merges any saved profile for that directory with flags you pass on the command line, and copies the result to the clipboard unless you use `--out`.

### Subcommands


| Command               | Description                                                                             |
| --------------------- | --------------------------------------------------------------------------------------- |
| `yoink`               | Walk the tree, concatenate text files, send output to the clipboard or `--out`.         |
| `yoink manage [PATH]` | Open the terminal UI to manage profiles and directory bindings. `PATH` defaults to `.`. |


### Options (default `yoink` run)


| Short | Long               | Description                                                      |
| ----- | ------------------ | ---------------------------------------------------------------- |
|       | `PATH`             | Root directory to scan. Default: `.`                             |
| `-e`  | `--exclude`        | Comma-separated paths (dirs or files) to skip.                   |
| `-i`  | `--include-only`   | Comma-separated glob patterns; only matching files are included. |
| `-H`  | `--include-hidden` | Comma-separated hidden paths to include explicitly.              |
|       | `--max-file-size`  | Maximum file size in bytes (default: 10485760).                  |
|       | `--depth`          | Maximum directory traversal depth.                               |
| `-o`  | `--out`            | Write to this file instead of the clipboard.                     |
| `-p`  | `--profile`        | Use this profile by name, ignoring directory bindings.           |
|       | `--save-profile`   | Save current flag set as a named profile for this project path.  |
|       | `--bind-profile`   | Bind the resolved project path to an existing profile name.      |


`--save-profile` may be combined with `--bind-profile` in one invocation. Profile and binding data are stored in a user-level configuration file.

## Contributing

Contributions are welcome.

1. Open an issue to describe a bug or propose a change, or comment on an existing issue, so work stays aligned with maintainers.
2. Fork the repository, create a branch from `master`, and make focused commits with clear messages.
3. Run `cargo fmt` and `cargo clippy` before opening a pull request. Fix any warnings or errors introduced by your changes.
4. Open a pull request that explains what changed and why. Link related issues when applicable.

If you are unsure whether a larger feature fits the project, open an issue first and outline the approach.

## License

See [LICENSE](LICENSE).