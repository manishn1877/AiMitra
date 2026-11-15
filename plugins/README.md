# Plugins

Plugins extend AiMitra's functionality.

## Built-in Plugins

| Plugin | Description |
|--------|-------------|
| shell | Execute shell commands |
| fs | File system operations |
| dialog | File dialogs |
| opener | Open URLs/files with default app |
| process | Process management |
| os | OS information |
| clipboard | Clipboard access |
| notification | Desktop notifications |
| updater | Auto-updates |
| ponytail | Enhanced plugin support |

## Creating Custom Plugins

Plugins are Rust libraries that expose commands to the frontend.

### Structure

```
plugins/my-plugin/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Example Plugin

```rust
use tauri::plugin::Plugin;

#[tauri::plugin]
pub struct MyPlugin(tauri::AppHandle);

#[tauri::plugin]
impl<T: RawDahandle> Plugin<T> for MyPlugin {
    fn initialize(&mut self, handle: &T) -> Result<(), tauri::Error> {
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn plugin_init(handle: RawDahandle) -> Result<RawDahandle, Box<dyn std::error::Error>> {
    // Initialize plugin
    Ok(handle)
}
```

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-my-plugin = "1.0"
```

Then enable in setup:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_my_plugin::init())
```