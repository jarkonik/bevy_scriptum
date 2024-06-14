# Live-reload

## Bevy included support

To enable life reload it should be enough to enable `file-watcher` feature
within bevy dependency in `Cargo.toml`

```
bevy = { version = "0.13", features = ["file_watcher"] }
```

## Init-teardown pattern for game development
