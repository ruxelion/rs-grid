import fs from "node:fs/promises";
import { SKILL_PATH } from "./paths.js";

export interface GridCommandInfo {
  name: string;
  category: string;
  signature: string;
  note?: string;
}

// Every GridCommand variant is documented on the same reference page.
export const COMMAND_DOC_PAGE = "api/grid-command";

let cache: GridCommandInfo[] | null = null;

// Parses one `#### Category` rust code block into individual command entries.
// Handles multi-line variants (struct-style payloads spanning several lines)
// by tracking brace depth, and captures trailing `// comments` as notes.
function parseEntries(code: string, category: string): GridCommandInfo[] {
  const cmds: GridCommandInfo[] = [];
  let curr: string[] = [];
  let notes: string[] = [];
  let depth = 0;

  const flush = () => {
    if (curr.length === 0) return;
    const signature = curr.join(" ").replace(/\s+/g, " ").trim();
    const nameMatch = signature.match(/^([A-Za-z]\w*)/);
    if (nameMatch) {
      cmds.push({
        name: nameMatch[1],
        category,
        signature,
        note: notes.length ? notes.join(" ") : undefined,
      });
    }
    curr = [];
    notes = [];
  };

  for (const raw of code.split("\n")) {
    if (!raw.trim()) continue;

    // Separate code from any trailing line comment.
    let codePart = raw;
    let comment = "";
    const ci = raw.indexOf("//");
    if (ci !== -1) {
      codePart = raw.slice(0, ci);
      comment = raw.slice(ci + 2).trim();
    }
    codePart = codePart.trim();

    // A new entry starts when we are outside braces and the line begins with
    // an identifier (variant names are PascalCase; struct fields are not).
    // Flush the previous entry *before* attaching this line's comment, so a
    // trailing comment belongs to the entry it sits on — not the previous one.
    if (depth === 0 && curr.length > 0 && /^[A-Z]\w*/.test(codePart)) {
      flush();
    }

    if (codePart) curr.push(codePart);
    if (comment) notes.push(comment);

    for (const ch of codePart) {
      if (ch === "{") depth++;
      else if (ch === "}") depth--;
    }
  }

  flush();
  return cmds;
}

function parseCommands(skill: string): GridCommandInfo[] {
  const anchor = "### Mutate grid state via GridCommand";
  const start = skill.indexOf(anchor);
  if (start === -1) return [];

  // The section runs until the next `### ` heading (same level) or EOF.
  const after = skill.slice(start + anchor.length);
  const nextHeading = after.search(/\n### /);
  const section =
    nextHeading === -1 ? after : after.slice(0, nextHeading);

  const commands: GridCommandInfo[] = [];
  const catRegex = /####\s+(.+?)\n([\s\S]*?)(?=####\s+|$)/g;
  let m: RegExpExecArray | null;
  while ((m = catRegex.exec(section)) !== null) {
    const category = m[1].trim();
    const codeMatch = m[2].match(/```rust\n([\s\S]*?)```/);
    if (!codeMatch) continue;
    commands.push(...parseEntries(codeMatch[1], category));
  }
  return commands;
}

export async function loadCommands(): Promise<GridCommandInfo[]> {
  if (cache) return cache;
  const skill = await fs.readFile(SKILL_PATH, "utf-8");
  cache = parseCommands(skill);
  return cache;
}
