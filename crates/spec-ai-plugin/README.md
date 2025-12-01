# spec-ai-plugin

Plugin system for custom tools in spec-ai.

## Overview

This crate provides the infrastructure for loading and running custom tools implemented as dynamic libraries (`.dylib` on macOS, `.so` on Linux, `.dll` on Windows).

- **ABI-Stable Interface**: Uses `abi_stable` for safe cross-binary compatibility
- **Plugin Loader**: Discovers and loads plugins from directories
- **Tool Abstraction**: Common interface for plugin-provided tools

## Features

- `plugin-api` - Minimal dependencies for plugin authors

## For Plugin Authors

To create a plugin, add this crate as a dependency with the `plugin-api` feature:

```toml
[dependencies]
spec-ai-plugin = { version = "0.4", features = ["plugin-api"] }
```

Then implement your tools using the ABI-stable types:

```rust
use abi_stable::std_types::{RStr, RString, RVec};
use spec_ai_plugin::{
    PluginModule, PluginModuleRef, PluginTool, PluginToolInfo,
    PluginToolRef, PluginToolResult, PLUGIN_API_VERSION,
};

// Define your tool
extern "C" fn my_tool_info() -> PluginToolInfo {
    PluginToolInfo::new(
        "my_tool",
        "Description of my tool",
        r#"{"type": "object", "properties": {}}"#,
    )
}

extern "C" fn my_tool_execute(args_json: RStr<'_>) -> PluginToolResult {
    PluginToolResult::success("Tool executed successfully")
}

static MY_TOOL: PluginTool = PluginTool {
    info: my_tool_info,
    execute: my_tool_execute,
    initialize: None,
};

// Export the plugin module
extern "C" fn api_version() -> u32 { PLUGIN_API_VERSION }
extern "C" fn plugin_name() -> RString { RString::from("my-plugin") }
extern "C" fn get_tools() -> RVec<PluginToolRef> {
    RVec::from(vec![&MY_TOOL])
}

#[abi_stable::export_root_module]
fn get_library() -> PluginModuleRef {
    PluginModuleRef::from_prefix(PluginModule {
        api_version,
        plugin_name,
        get_tools,
        shutdown: None,
    })
}
```

## For Host Applications

Use the `PluginLoader` to discover and load plugins from a directory:

```rust
use spec_ai_plugin::{PluginLoader, expand_tilde};
use std::path::Path;

let mut loader = PluginLoader::new();
let stats = loader.load_directory(&expand_tilde(Path::new("~/.spec-ai/tools")))?;

println!("Loaded {} plugins with {} tools", stats.loaded, stats.tools_loaded);

for (tool, plugin_name) in loader.all_tools() {
    let info = (tool.info)();
    println!("  - {} from {}", info.name, plugin_name);
}
```

## Usage

This is an internal crate primarily used by:
- `spec-ai-core` - For loading and executing custom tools

For end-user documentation, see the main [spec-ai README](../../README.md).
