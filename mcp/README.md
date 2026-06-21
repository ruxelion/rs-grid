# rs-grid-mcp

[Model Context Protocol](https://modelcontextprotocol.io) server for
[**rs-grid**](https://rs-grid.com) — a headless, high-performance data grid for
Rust → WASM (Leptos / Dioxus / Yew, or framework-free).

It gives AI agents two things they otherwise lack when working on rs-grid:

1. **The documentation** — searchable, with per-type lookup.
2. **The rendered scene** — rs-grid draws to `<canvas>`, so the DOM reveals
   nothing about the output. This server serves serialized `SceneFrame`s (the
   renderer-agnostic primitives the grid would actually draw) so an agent can
   reason about the rendering without a browser.

## Install

Runs over stdio via `npx`; no global install needed.

### Claude Code

```sh
claude mcp add rs-grid -- npx -y rs-grid-mcp
```

### Other MCP clients

```jsonc
{
  "mcpServers": {
    "rs-grid": {
      "command": "npx",
      "args": ["-y", "rs-grid-mcp"]
    }
  }
}
```

## Tools

| Tool | Description |
| --- | --- |
| `search_rs_grid_docs` | Keyword search across the docs; returns relevant excerpts. `language`: `en` (default) / `fr`. |
| `get_api_type` | Full doc page for a type by name (e.g. `GridCommand`, `ColumnDef`, `CellFormat`, `GridState`, `Theme`). |
| `list_doc_pages` | List doc pages, optionally filtered by section (`api`, `features`, `concepts`, `data`, `theming`, `integrations`, `scene`, `deployment`, `development`). |
| `list_scenes` | List the canonical render scenarios available to `get_scene`. |
| `get_scene` | Get a scenario's `SceneFrame` as JSON — every primitive's geometry, color, clip and text. Scenarios: `basic`, `selection`, `pinned`, `scrolled`. |

### Inspecting the render

`get_scene` is the differentiator. Each scenario returns the exact primitives
the canvas renderer would draw — letting an agent verify, for example, that a
pinned column lands at the right offset, or that a selected cell is highlighted
with the right fill — purely from data:

```jsonc
// get_scene { "name": "pinned" }  →
{
  "primitives": [
    { "Rect": { "x": 0, "y": 0, "width": 800, "height": 400, "fill": { "r": 255, "g": 255, "b": 255, "a": 255 }, ... } },
    { "Text": { "x": 54, "y": 59.9, "text": "r0c0", "color": { ... }, "clip": [42, 40, 120, 30], ... } }
    // ...
  ],
  "button_zones": [],
  "viewport_width": 800,
  "viewport_height": 400,
  "dpr": 1
}
```

The fixtures are generated from the same scenarios that back rs-grid's snapshot
tests, so they stay faithful to the real renderer.

## Resources

| URI | Description |
| --- | --- |
| `rs-grid://llms.txt` | Documentation index. |
| `rs-grid://llms-full.txt` | Full concatenated documentation. |
| `rs-grid://skill.md` | Skill definition (capabilities, constraints, workflows). |
| `rs-grid://docs/{path}` | Any individual documentation page. |

## Links

- Docs: <https://rs-grid.com>
- Project: <https://ruxelion.com>

## License

See the rs-grid repository.
