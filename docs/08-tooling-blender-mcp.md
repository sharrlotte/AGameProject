# Tooling: Blender MCP

## 1. Purpose

Blender is used for character/rig setup, animation authoring, and glTF export for the player model.
This document records the MCP setup so the connection can be reproduced or repaired later.

The MCP implementation is **`mcp-for-blender`** (the renamed successor to `blender-mcp`).

## 2. Installed Components

| Component | Version | Location |
| :--- | :--- | :--- |
| Blender | 5.3.0 | `C:\Program Files\Blender Foundation\Blender 5.3\blender.exe` |
| uv / uvx | 0.13.7 | `C:\Users\nguyen\.local\bin\uvx.exe` |
| MCP server | `mcp-for-blender` (via `uvx`) | Cached by uv; no global install needed |
| Blender addon | `blender_mcp.py` | `%APPDATA%\Blender Foundation\Blender\5.3\scripts\addons\blender_mcp.py` |

The addon is enabled in Blender's user preferences. The MCP server talks to the addon over
`localhost:9876`.

## 3. Client Configuration

### OpenCode

File: `~/.config/opencode/opencode.json`

```json
{
  "mcp": {
    "blender-mcp": {
      "type": "local",
      "command": [
        "C:\\Users\\nguyen\\.local\\bin\\uvx.exe",
        "--python",
        "3.11",
        "mcp-for-blender"
      ],
      "enabled": true,
      "environment": {
        "BLENDER_HOST": "localhost",
        "BLENDER_PORT": "9876"
      }
    }
  }
}
```

The absolute `uvx.exe` path is intentional. GUI clients do not always inherit the user PATH.

### Antigravity

File: `~/.gemini/config/mcp_config.json`

```json
{
  "mcpServers": {
    "blender-mcp": {
      "command": "C:\\Users\\nguyen\\.local\\bin\\uvx.exe",
      "args": ["--python", "3.11", "mcp-for-blender"],
      "env": {
        "BLENDER_HOST": "localhost",
        "BLENDER_PORT": "9876"
      }
    }
  }
}
```

The existing `solim` server entry in that file is preserved.

## 4. Starting the Connection

The addon is installed and enabled, but the server is started manually for safety.

1. Open or restart Blender.
2. In the 3D viewport, press `N` to open the sidebar.
3. Open the **MCP for Blender** tab.
4. Click **Connect to MCP server**.
5. Confirm the connection from OpenCode with the `blender-mcp` `get_addon_status` tool.

Do not enable auto-start in a `.blend` or in the startup file. The addon can execute arbitrary
Python inside Blender, so the port should only be open while actively working.

## 5. Verification

```powershell
# uv
& "$env:USERPROFILE\.local\bin\uvx.exe" --version

# OpenCode server registration
opencode mcp list

# Installer / addon detection
& "$env:USERPROFILE\.local\bin\uvx.exe" --python 3.11 mcp-for-blender addon-paths
```

`opencode mcp list` should show `blender-mcp` as connected. The Blender addon status only becomes
available after step 4 above.

## 6. Security Notes

- The addon exposes arbitrary Python execution and has no authentication or encryption.
- Keep `BLENDER_HOST=localhost`; do not expose port `9876` to a network.
- Start the server manually per session.
- The `uvx` cache and the addon are third-party software. Review updates before applying them.

## 7. Troubleshooting

| Symptom | Fix |
| :--- | :--- |
| OpenCode shows `spawn uvx ENOENT` | Use the absolute `uvx.exe` path in the config. |
| `Could not connect to Blender` | The addon server is not running; click **Connect to MCP server** in Blender. |
| Blender addon missing from Preferences | Restart Blender, or run `uvx --python 3.11 mcp-for-blender install-addon` again. |
| Antigravity does not see the server | Restart Antigravity after editing `mcp_config.json`. |
| Port already in use | Stop the other MCP client or Blender addon instance; only one server should own port `9876`. |

## 8. Related Decisions

- Blender work is agent-assisted through this MCP and/or headless Blender Python scripts.
- Mixamo assets are downloaded by the project owner; the agent imports, retargets, and exports them.
- Hand IK runs at runtime in Bevy; Blender bakes only the base motion.
- First animation milestone: reload while walking, rifle only, one generic reload.
- Visual target: stylized low-poly.
