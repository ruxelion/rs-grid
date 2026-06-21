// Maps type names (case-insensitive, no whitespace) to relative doc page
// paths inside doc_build/. Used by the get_api_type tool.
export const TYPE_TO_DOC_PATH: Record<string, string> = {
  // API reference
  gridcommand: "api/grid-command",
  commandoutput: "api/grid-command",
  copyerror: "api/grid-command",
  gridstate: "api/grid-state",
  gridmodel: "api/grid-model",
  columndef: "api/column-def",
  cellformat: "api/column-def",
  celleditor: "api/column-def",
  cellvalidator: "api/column-def",
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
