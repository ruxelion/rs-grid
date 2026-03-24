import { useEffect, useRef, useState } from 'react'
import { useGrid } from './hooks/useGrid'

function fmtRows(n: number): string {
  switch (n) {
    case 1_000: return '1 000 rows'
    case 100_000: return '100 000 rows'
    case 1_000_000: return '1 million rows'
    case 100_000_000: return '100 million rows'
    case 1_000_000_000: return '1 billion rows'
    case 1_000_000_000_000: return '1 trillion rows'
    case 1_000_000_000_000_000: return '1 quadrillion rows'
    default: return 'rows'
  }
}

function fmtCols(n: number): string {
  switch (n) {
    case 20: return '20 columns'
    case 100: return '100 columns'
    case 1000: return '1 000 columns'
    default: return 'columns'
  }
}

export default function App() {
  const [rowCount, setRowCount] = useState(1_000)
  const [colCount, setColCount] = useState(20)
  const [themeClass, setThemeClass] = useState('')
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const gridRef = useGrid(canvasRef, rowCount, colCount)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Apply theme class to <html> and refresh grid colours
  useEffect(() => {
    document.documentElement.className = themeClass
    gridRef.current?.set_theme_from_css()
  }, [themeClass, gridRef])

  function handleExport() {
    const gc = gridRef.current
    if (!gc) return
    const data = gc.export_patches()
    const encoded = encodeURIComponent(data)
    const url = `data:text/tab-separated-values;charset=utf-8,${encoded}`
    const a = document.createElement('a')
    a.href = url
    a.download = 'rs-grid-patches.tsv'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
  }

  function handleImportChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onloadend = () => {
      const text = reader.result
      if (typeof text === 'string') {
        gridRef.current?.import_patches(text)
        try {
          localStorage.setItem('rs-grid-patches', gridRef.current?.export_patches() ?? '')
        } catch { /* ignore */ }
      }
    }
    reader.readAsText(file)
  }

  return (
    <main className="app-layout">
      <div className="app-page-header">
        <h1 className="app-title">rs-grid basic example</h1>
        <p className="app-subtitle">
          {'Use the '}
          <strong className="app-highlight">{fmtRows(rowCount)}</strong>
          {' × '}
          <strong className="app-highlight">{fmtCols(colCount)}</strong>
          {' virtual dataset below to test windowed rendering.'}
        </p>
        <div className="app-controls">

          {/* Dataset size */}
          <div className="app-control">
            <span className="app-control-label">Dataset size</span>
            <select
              className="app-control-select"
              value={rowCount}
              onChange={e => setRowCount(Number(e.target.value))}
            >
              <option value={1_000}>1 000 rows</option>
              <option value={100_000}>100 000 rows</option>
              <option value={1_000_000}>1 million rows</option>
              <option value={100_000_000}>100 million rows</option>
              <option value={1_000_000_000}>1 billion rows</option>
              <option value={1_000_000_000_000}>1 trillion rows</option>
              <option value={1_000_000_000_000_000}>1 quadrillion rows</option>
            </select>
          </div>

          {/* Column count */}
          <div className="app-control">
            <span className="app-control-label">Column count</span>
            <select
              className="app-control-select"
              value={colCount}
              onChange={e => setColCount(Number(e.target.value))}
            >
              <option value={20}>20 columns</option>
              <option value={100}>100 columns</option>
              <option value={1000}>1 000 columns</option>
            </select>
          </div>

          {/* Export */}
          <button className="app-btn" onClick={handleExport}>Export</button>

          {/* Hidden file input for import */}
          <input
            ref={fileInputRef}
            type="file"
            accept=".tsv,.txt"
            style={{ display: 'none' }}
            onChange={handleImportChange}
          />

          {/* Import */}
          <button className="app-btn" onClick={() => fileInputRef.current?.click()}>
            Import
          </button>

          {/* Pinned cols */}
          <div className="app-control">
            <span className="app-control-label">Pinned cols</span>
            <select
              className="app-control-select"
              defaultValue={0}
              onChange={e => gridRef.current?.set_pinned_count(Number(e.target.value))}
            >
              <option value={0}>None</option>
              <option value={1}>1</option>
              <option value={2}>2</option>
              <option value={3}>3</option>
            </select>
          </div>

          {/* Filter Name */}
          <div className="app-control">
            <span className="app-control-label">Filter Name</span>
            <input
              type="text"
              className="app-control-select"
              placeholder="type to filter…"
              onInput={e =>
                gridRef.current?.set_filter('name', (e.target as HTMLInputElement).value)
              }
            />
          </div>

          {/* Theme */}
          <div className="app-control">
            <span className="app-control-label">Theme</span>
            <select
              className="app-control-select"
              value={themeClass}
              onChange={e => setThemeClass(e.target.value)}
            >
              <option value="">Light</option>
              <option value="dark">Dark</option>
              <option value="material">Material 3</option>
              <option value="material-dark">Material 3 Dark</option>
            </select>
          </div>

        </div>
      </div>

      {/* Grid canvas */}
      <div className="app-body">
        <div className="app-grid-wrapper">
          <canvas
            ref={canvasRef}
            style={{ width: '100%', height: '100%', display: 'block' }}
          />
        </div>
      </div>
    </main>
  )
}
