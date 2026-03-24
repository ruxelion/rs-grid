import { useEffect, useRef } from 'react'
import type { JsGrid } from 'basic-js'

const LS_KEY = 'rs-grid-patches'

function localStorageGet(): string | null {
  try { return localStorage.getItem(LS_KEY) } catch { return null }
}
function localStorageSet(v: string) {
  try { localStorage.setItem(LS_KEY, v) } catch { /* ignore */ }
}

export function useGrid(
  canvasRef: React.RefObject<HTMLCanvasElement | null>,
  rowCount: number,
  colCount: number,
) {
  const gridRef = useRef<JsGrid | null>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    let cancelled = false

    import('basic-js').then(async ({ default: init, JsGrid }) => {
      await init()
      if (cancelled) return
      gridRef.current?.detach()
      const grid = new JsGrid(canvas, rowCount, colCount)
      const saved = localStorageGet()
      if (saved) grid.import_patches(saved)
      grid.set_on_change(() => localStorageSet(grid.export_patches()))
      gridRef.current = grid
    })

    return () => {
      cancelled = true
      gridRef.current?.detach()
      gridRef.current = null
    }
  }, [rowCount, colCount]) // eslint-disable-line react-hooks/exhaustive-deps

  return gridRef
}
