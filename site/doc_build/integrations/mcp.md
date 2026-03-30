# MCP Server

## Overview

rs-grid provides a **Model Context Protocol (MCP) server** published on npm as
[`rs-grid-mcp`](https://www.npmjs.com/package/rs-grid-mcp). It is a TypeScript
stdio server that exposes the full documentation to Claude Code and any other
MCP-compatible AI agent.

Once registered, the agent can search the documentation, read individual pages,
and access the full context index — without leaving the conversation.

## Setup

### In your own project (npx)

No installation required. Add a `.mcp.json` file at the root of your project:

```json title=".mcp.json"
{
  "mcpServers": {
    "rs-grid-docs": {
      "command": "npx",
      "args": ["-y", "rs-grid-mcp"]
    }
  }
}
```

Claude Code auto-detects `.mcp.json` on project open and asks for a one-time
approval. `npx -y` downloads and runs the latest version automatically.

For **VS Code Copilot**, create `.vscode/mcp.json` instead:

```json title=".vscode/mcp.json"
{
  "servers": {
    "rs-grid-docs": {
      "command": "npx",
      "args": ["-y", "rs-grid-mcp"],
      "type": "stdio"
    }
  }
}
```

**CLI registration** (user-level, all projects):

```bash
claude mcp add rs-grid-docs -- npx -y rs-grid-mcp
```

### In the rs-grid repository (contributors)

The repo contains a pre-configured `.mcp.json` pointing to the local build.
Build the server once, then Claude Code picks it up automatically:

```bash
just mcp-build
```

:::tip
When working on the MCP source itself, use `just mcp-dev` to run via `tsx`
without a build step — changes take effect on the next server restart.
:::

## Available tools

| Tool                  | Parameters                                                                                   | Description                                                                                        |
| --------------------- | -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `search_rs_grid_docs` | `query` (string), `limit` (number, default 5), `language` (`"en"` \| `"fr"`, default `"en"`) | Keyword search across all documentation pages. Returns scored excerpts centred on the first match. |

## Available resources

| URI                       | Description                                               |
| ------------------------- | --------------------------------------------------------- |
| `rs-grid://llms.txt`      | Documentation index — all page titles and descriptions    |
| `rs-grid://llms-full.txt` | Full documentation concatenated (\~5 000 lines)           |
| `rs-grid://skill.md`      | Capabilities, constraints, and workflows for AI agents    |
| `rs-grid://docs/{path}`   | Individual page — e.g. `rs-grid://docs/api/grid-state.md` |
