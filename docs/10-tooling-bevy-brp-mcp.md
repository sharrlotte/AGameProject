# Tooling: Bevy BRP MCP

## 1. Purpose

The Bevy game runs with the Bevy Remote Protocol enabled (`bevy::remote::RemotePlugin` and
`bevy_brp_extras::BrpExtrasPlugin`). The `bevy_brp_mcp` MCP server bridges that running app to the
agent so the game's live state and world can be inspected and driven.

## 2. Installed Components

| Component | Location |
| :--- | :--- |
| MCP server | `bevy_brp_mcp` | `C:\Users\nguyen\.cargo\bin\bevy_brp_mcp.exe` |

The server is a stdio MCP server that connects to the running Bevy app over the BRP transport.

## 3. Client Configuration

### OpenCode

File: `~/.config/opencode/opencode.json`

```json
{
    "mcp": {
        "brp": {
            "type": "local",
            "command": [
                "C:\\Users\\nguyen\\.cargo\\bin\\bevy_brp_mcp.exe"
            ],
            "enabled": true,
            "environment": {}
        }
    }
}
```

The absolute `bevy_brp_mcp.exe` path is intentional. GUI clients do not always inherit the user PATH.

## 4. Starting the Connection

1. Start the Bevy app with the remote protocol enabled.
2. Confirm the BRP port (default `15702`) is reachable.
3. From OpenCode, use the `brp_*` tools (for example `rpc_discover` or `brp_list_agent_tools`) to talk
   to the running app.

## 5. Verification

```powershell
opencode mcp list
```

`brp` should be listed as connected once the Bevy app is running.

## 6. Security Notes

- Keep the BRP listener on localhost only.
- Start the game only while actively working.

## 7. Related Decisions

- Bevy work is agent-assisted through this MCP and/or headless Bevy Python scripts.
- Mixamo assets are downloaded by the project owner; the agent imports, retargets, and exports them.
- Hand IK runs at runtime in Bevy; Blender bakes only the base motion.
