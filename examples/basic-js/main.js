import init, { JsGrid } from "./pkg/basic_js.js";

const ROW_LABELS = {
  1000: "1 000 rows",
  100000: "100 000 rows",
  1000000: "1 million rows",
  100000000: "100 million rows",
  1000000000: "1 billion rows",
  1000000000000: "1 trillion rows",
  1000000000000000: "1 quadrillion rows",
};

const COL_LABELS = {
  20: "20 columns",
  100: "100 columns",
  1000: "1 000 columns",
};

async function main() {
  await init();

  const canvas = document.getElementById("grid-canvas");
  let rowCount = 1000;
  let colCount = 20;
  let grid = new JsGrid(canvas, rowCount, colCount);

  // ── Row count ────────────────────────────────────
  document.getElementById("row-select").addEventListener("change", (e) => {
    rowCount = Number(e.target.value);
    document.getElementById("row-label").textContent =
      ROW_LABELS[rowCount] || `${rowCount} rows`;
    grid.detach();
    grid = new JsGrid(canvas, rowCount, colCount);
  });

  // ── Column count ─────────────────────────────────
  document.getElementById("col-select").addEventListener("change", (e) => {
    colCount = Number(e.target.value);
    document.getElementById("col-label").textContent =
      COL_LABELS[colCount] || `${colCount} columns`;
    grid.detach();
    grid = new JsGrid(canvas, rowCount, colCount);
  });

  // ── Export ────────────────────────────────────────
  document.getElementById("btn-export").addEventListener("click", () => {
    const data = grid.export_patches();
    const encoded = encodeURIComponent(data);
    const url = `data:text/tab-separated-values;charset=utf-8,${encoded}`;
    const a = document.createElement("a");
    a.href = url;
    a.download = "rs-grid-patches.tsv";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  });

  // ── Import ────────────────────────────────────────
  const fileInput = document.getElementById("file-input");
  document.getElementById("btn-import").addEventListener("click", () => {
    fileInput.click();
  });
  fileInput.addEventListener("change", () => {
    const file = fileInput.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onloadend = () => {
      if (typeof reader.result === "string") {
        grid.import_patches(reader.result);
      }
    };
    reader.readAsText(file);
    fileInput.value = "";
  });

  // ── Pinned columns ───────────────────────────────
  document.getElementById("pinned-select").addEventListener("change", (e) => {
    grid.set_pinned_count(Number(e.target.value));
  });

  // ── Filter ────────────────────────────────────────
  document.getElementById("filter-input").addEventListener("input", (e) => {
    grid.set_filter("name", e.target.value);
  });

  // ── Dark mode ─────────────────────────────────────
  document.getElementById("dark-toggle").addEventListener("change", (e) => {
    const dark = e.target.checked;
    document.documentElement.classList.toggle("dark", dark);
    document.getElementById("dark-label").textContent = dark
      ? "Light mode"
      : "Dark mode";
    grid.set_theme_from_css();
  });
}

main();
