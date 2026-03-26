import {
  McpServer,
  ResourceTemplate,
} from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import fs from "node:fs/promises";
import { z } from "zod";
import {
  DOC_BUILD_ROOT,
  SKILL_PATH,
  resolveDocPath,
} from "./paths.js";
import { searchDocs } from "./search.js";

const server = new McpServer({
  name: "rs-grid-docs",
  version: "0.1.0",
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
