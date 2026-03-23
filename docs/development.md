# Development

## Run Tests

```bash
cargo test
```

## Run the App From the Repository

```bash
cargo run --
```

## Use a Temporary Config Directory

```bash
cargo run -- -C /tmp/wkey-demo i -y
cargo run -- -C /tmp/wkey-demo s c -g shell copy -k Ctrl+C -d "Copy selection"
cargo run -- -C /tmp/wkey-demo
```

## Related Docs

- For end-user commands, see [CLI reference](cli-reference.md).
- For config layout, see [Configuration](configuration.md).
