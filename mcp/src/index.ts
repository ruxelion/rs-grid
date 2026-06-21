#!/usr/bin/env node
import {
  McpServer,
  ResourceTemplate,
} from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import fs from "node:fs/promises";
import path from "node:path";
import { z } from "zod";
import {
  DOC_BUILD_ROOT,
  SCENES_ROOT,
  SKILL_PATH,
  resolveDocPath,
} from "./paths.js";
import { searchDocs, loadDocs } from "./search.js";
import { TYPE_TO_DOC_PATH, SECTION_PREFIXES } from "./type-map.js";

const server = new McpServer({
  name: "rs-grid-docs",
  version: "0.2.2",
});

// --- Tool: search_rs_grid_docs ---

server.tool(
  "search_rs_grid_docs",
  "Search rs-grid documentation by keywords. Returns relevant excerpts from the documentation.",
  {
    query: z
      .string()
      .describe("Search terms (space-separated keywords)"),
    limit: z
      .number()
      .optional()
      .default(5)
      .describe("Maximum number of results (default 5)"),
    language: z
      .enum(["en", "fr"])
      .optional()
      .default("en")
      .describe("Documentation language: en or fr"),
  },
  async ({ query, limit, language }) => {
    const result = await searchDocs(query, limit, language);
    return { content: [{ type: "text", text: result }] };
  },
);

// --- Tool: get_api_type ---

server.tool(
  "get_api_type",
  "Look up the documentation page for a specific rs-grid type by name. " +
    "Returns the full markdown content of the relevant doc page. " +
    "Example type names: GridCommand, ColumnDef, CellFormat, DataSource, " +
    "VecDataSource, GridState, Theme, Locale, SortState, SelectionState.",
  {
    type_name: z
      .string()
      .describe("Type name to look up (case-insensitive, e.g. 'GridCommand')"),
    language: z
      .enum(["en", "fr"])
      .optional()
      .default("en")
      .describe("Documentation language: en or fr"),
  },
  async ({ type_name, language }) => {
    const key = type_name.toLowerCase().replace(/\s+/g, "");
    const docPath = TYPE_TO_DOC_PATH[key];

    if (!docPath) {
      const known = Object.keys(TYPE_TO_DOC_PATH)
        .map((k) => TYPE_TO_DOC_PATH[k])
        .filter((v, i, a) => a.indexOf(v) === i)
        .sort()
        .join(", ");
      return {
        content: [
          {
            type: "text",
            text:
              `Unknown type: "${type_name}". ` +
              `Known doc pages: ${known}. ` +
              `Try search_rs_grid_docs for a keyword search instead.`,
          },
        ],
      };
    }

    const resolvedPath =
      language === "fr"
        ? resolveDocPath(`fr/${docPath}.md`)
        : resolveDocPath(`${docPath}.md`);

    if (!resolvedPath.startsWith(DOC_BUILD_ROOT)) {
      return {
        content: [{ type: "text", text: "Error: invalid path." }],
      };
    }

    const content = await fs.readFile(resolvedPath, "utf-8");
    return { content: [{ type: "text", text: content }] };
  },
);

// --- Tool: list_doc_pages ---

server.tool(
  "list_doc_pages",
  "List available documentation pages, optionally filtered by section. " +
    "Returns page titles, paths, and one-line descriptions. " +
    "Sections: api, features, concepts, data, theming, integrations, " +
    "scene, deployment, development. Omit section to list all pages.",
  {
    section: z
      .enum([
        "api",
        "features",
        "concepts",
        "data",
        "theming",
        "integrations",
        "scene",
        "deployment",
        "development",
      ])
      .optional()
      .describe("Filter by section (optional)"),
    language: z
      .enum(["en", "fr"])
      .optional()
      .default("en")
      .describe("Documentation language: en or fr"),
  },
  async ({ section, language }) => {
    const docs = await loadDocs(language);

    const prefix = section ? SECTION_PREFIXES[section] : null;
    const filtered = prefix
      ? docs.filter((d) => d.path.startsWith(prefix))
      : docs;

    if (filtered.length === 0) {
      return {
        content: [
          {
            type: "text",
            text: section
              ? `No pages found in section "${section}".`
              : "No documentation pages found.",
          },
        ],
      };
    }

    const lines: string[] = [
      `## ${filtered.length} page(s)${section ? ` in "${section}"` : ""}\n`,
    ];
    for (const doc of filtered.sort((a, b) => a.path.localeCompare(b.path))) {
      lines.push(`- **${doc.path}** — ${doc.title}`);
    }

    return { content: [{ type: "text", text: lines.join("\n") }] };
  },
);

// --- Tool: list_scenes / get_scene ---
//
// rs-grid renders to <canvas>, so the DOM tells you nothing about the output.
// These tools expose canonical SceneFrames — the renderer-agnostic primitives
// (rects, text, lines; positions, colors, clips) the grid would actually draw —
// so an agent can reason about the rendering without a browser. Fixtures are
// generated from the same scenarios that back the insta snapshot tests
// (`just gen-scene-fixtures`).

const SCENES: { name: string; description: string }[] = [
  { name: "basic", description: "5×20 grid, default view" },
  { name: "selection", description: "cell (row 2, col 1) selected" },
  { name: "pinned", description: "first 2 columns pinned" },
  { name: "scrolled", description: "10×200 grid scrolled to x=300, y=600" },
];

server.tool(
  "list_scenes",
  "List the canonical rs-grid render scenarios available via get_scene. Each is " +
    "a serialized SceneFrame: the renderer-agnostic primitives the grid would draw.",
  {},
  async () => {
    const lines = ["## rs-grid render scenarios\n"];
    for (const s of SCENES) lines.push(`- **${s.name}** — ${s.description}`);
    lines.push(
      "\nFetch one with get_scene to inspect its exact rendered primitives.",
    );
    return { content: [{ type: "text", text: lines.join("\n") }] };
  },
);

server.tool(
  "get_scene",
  "Get a canonical rs-grid SceneFrame as JSON: every primitive's geometry, " +
    "color, clip and text for a scenario. Lets you verify what the grid renders " +
    "(e.g. a pinned column's offset) without a browser. See list_scenes for names.",
  {
    name: z
      .enum(["basic", "selection", "pinned", "scrolled"])
      .describe("Scenario name (see list_scenes)"),
  },
  async ({ name }) => {
    const filePath = path.join(SCENES_ROOT, `${name}.json`);
    // Defence in depth (the enum already constrains `name`).
    if (!filePath.startsWith(SCENES_ROOT)) {
      return { content: [{ type: "text", text: "Error: invalid path." }] };
    }
    const content = await fs.readFile(filePath, "utf-8");
    return { content: [{ type: "text", text: content }] };
  },
);

// --- Resource: llms.txt ---

server.resource(
  "llms-txt",
  "rs-grid://llms.txt",
  { description: "Documentation index for rs-grid (llms.txt)" },
  async () => {
    const content = await fs.readFile(
      resolveDocPath("llms.txt"),
      "utf-8",
    );
    return { contents: [{ uri: "rs-grid://llms.txt", text: content }] };
  },
);

// --- Resource: llms-full.txt ---

server.resource(
  "llms-full-txt",
  "rs-grid://llms-full.txt",
  {
    description:
      "Full concatenated documentation for rs-grid (llms-full.txt)",
  },
  async () => {
    const content = await fs.readFile(
      resolveDocPath("llms-full.txt"),
      "utf-8",
    );
    return {
      contents: [
        { uri: "rs-grid://llms-full.txt", text: content },
      ],
    };
  },
);

// --- Resource: skill.md ---

server.resource(
  "skill-md",
  "rs-grid://skill.md",
  {
    description:
      "Skill definition for rs-grid (capabilities, constraints, workflows)",
  },
  async () => {
    const content = await fs.readFile(SKILL_PATH, "utf-8");
    return {
      contents: [{ uri: "rs-grid://skill.md", text: content }],
    };
  },
);

// --- Resource template: individual doc pages ---

server.resource(
  "doc-page",
  new ResourceTemplate("rs-grid://docs/{path}", { list: undefined }),
  { description: "Individual documentation page from doc_build/" },
  async (uri, variables) => {
    const docPath = variables.path as string;
    const filePath = resolveDocPath(docPath);

    // Prevent path traversal
    if (!filePath.startsWith(DOC_BUILD_ROOT)) {
      return {
        contents: [
          {
            uri: uri.href,
            text: "Error: invalid path.",
          },
        ],
      };
    }

    const content = await fs.readFile(filePath, "utf-8");
    return {
      contents: [{ uri: uri.href, text: content }],
    };
  },
);

// --- Start server ---

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("rs-grid-docs MCP server running on stdio");
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
