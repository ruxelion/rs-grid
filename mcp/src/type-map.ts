// Maps type names (case-insensitive, no whitespace) to relative doc page
// paths inside doc_build/. Used by the get_api_type tool.
export const TYPE_TO_DOC_PATH: Record<string, string> = {
  // API reference
  gridcommand: "api/grid-command",
  commandoutput: "api/grid-command",
  copyerror: "api/grid-command",
  gridstate: "api/grid-state",
  gridmodel: "api/grid-model",
  gridmodelbuilder: "api/grid-model",
  invalideditmode: "api/grid-model",
  columndef: "api/column-def",
  cellformat: "api/column-def",
  celleditor: "api/column-def",
  cellvalidator: "api/column-def",
  validationrule: "api/column-def",
  editablepredicate: "api/column-def",
  buttondef: "api/column-def",
  buttonstyle: "api/column-def",
  selectoption: "api/column-def",
  columnoffsets: "api/column-def",
  formattedcell: "api/column-def",
  cellalign: "api/column-def",
  cellelement: "api/column-def",
  theme: "api/theme",
  sceneprimitive: "api/scene-primitive",
  sceneframe: "api/scene-primitive",
  scenebuilder: "api/scene-primitive",
  rectprimitive: "api/scene-primitive",
  textprimitive: "api/scene-primitive",
  lineprimitive: "api/scene-primitive",
  polygonprimitive: "api/scene-primitive",
  imageprimitive: "api/scene-primitive",

  // Data sources
  datasource: "data/overview",
  cellstatus: "data/overview",
  datasourcemode: "data/overview",
  vecdatasource: "data/vec-datasource",
  rowrecord: "data/vec-datasource",
  cellvalue: "data/vec-datasource",
  fndatasource: "data/fn-datasource",
  pagecache: "data/page-cache",
  pagecachedatasource: "data/page-cache",
  pagefetchrequest: "data/page-cache",
  pagefetchresponse: "data/page-cache",
  fetchconfig: "data/page-cache",

  // Concepts
  selectionstate: "concepts/selection",
  cellcoord: "concepts/selection",
  viewportstate: "concepts/viewport",

  // Features
  searchstate: "features/search",
  editcell: "features/editing",
  sortstate: "features/sorting",
  sortdir: "features/sorting",
  scrollbargeom: "features/scrollbars",
  hscrollbargeom: "features/scrollbars",
  contextmenuconfig: "features/context-menu",
  contextmenuitem: "features/context-menu",
  builtinaction: "features/context-menu",
  locale: "features/localization",

  // Web / canvas
  gridcanvas: "api/grid-state",
};

export const SECTION_PREFIXES: Record<string, string> = {
  api: "api/",
  features: "features/",
  concepts: "concepts/",
  data: "data/",
  theming: "theming/",
  integrations: "integrations/",
  scene: "scene/",
  deployment: "deployment/",
  development: "development/",
};

// --- Type discovery helpers (used by get_api_type) ---

/** All mapped type names (normalized keys), sorted. */
export function knownTypeNames(): string[] {
  return Object.keys(TYPE_TO_DOC_PATH).sort();
}

/** Group type names by the doc page they resolve to. */
export function typesByPage(): Map<string, string[]> {
  const grouped = new Map<string, string[]>();
  for (const [type, page] of Object.entries(TYPE_TO_DOC_PATH)) {
    const arr = grouped.get(page) ?? [];
    arr.push(type);
    grouped.set(page, arr);
  }
  for (const arr of grouped.values()) arr.sort();
  return grouped;
}

function levenshtein(a: string, b: string): number {
  const m = a.length;
  const n = b.length;
  if (m === 0) return n;
  if (n === 0) return m;
  let prev = Array.from({ length: n + 1 }, (_, i) => i);
  let curr = new Array<number>(n + 1);
  for (let i = 1; i <= m; i++) {
    curr[0] = i;
    for (let j = 1; j <= n; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      curr[j] = Math.min(prev[j] + 1, curr[j - 1] + 1, prev[j - 1] + cost);
    }
    [prev, curr] = [curr, prev];
  }
  return prev[n];
}

/**
 * Suggest the closest known type names for a (mistyped) input. Substring
 * containment is treated as a strong signal; otherwise edit distance is used.
 */
export function suggestTypes(input: string, max = 5): string[] {
  const norm = input.toLowerCase().replace(/\s+/g, "");
  if (!norm) return [];
  const scored = Object.keys(TYPE_TO_DOC_PATH).map((key) => {
    let dist = levenshtein(norm, key);
    if (key.includes(norm) || norm.includes(key)) dist = Math.min(dist, 1);
    return { key, dist };
  });
  scored.sort((a, b) => a.dist - b.dist || a.key.localeCompare(b.key));
  return scored
    .filter((s) => s.dist <= 3)
    .slice(0, max)
    .map((s) => s.key);
}
