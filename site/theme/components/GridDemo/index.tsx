import { useState, useEffect, useRef, useCallback } from 'react';
import styles from './index.module.css';

interface GridDemoProps {
  t: (key: string) => string;
}

const ROW_OPTIONS = [
  { value: 1_000, label: '1K' },
  { value: 100_000, label: '100K' },
  { value: 1_000_000, label: '1M' },
] as const;

const COL_COUNT = 20;

export default function GridDemo({ t }: GridDemoProps) {
  const [status, setStatus] = useState<'idle' | 'loading' | 'ready' | 'error'>(
    'idle',
  );
  const [rowCount, setRowCount] = useState(1_000);

  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const gridRef = useRef<any>(null);
  const JsGridRef = useRef<any>(null);
  const resizeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const mountGrid = useCallback(
    (rows: number) => {
      const canvas = canvasRef.current;
      if (!canvas || !JsGridRef.current) return;
      gridRef.current?.detach();
      gridRef.current = new JsGridRef.current(canvas, rows, COL_COUNT);
    },
    [],
  );

  // Lazy load via IntersectionObserver
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          observer.disconnect();
          loadWasm();
        }
      },
      { threshold: 0.1, rootMargin: '200px' },
    );
    observer.observe(el);

    return () => observer.disconnect();
  }, []);

  async function loadWasm() {
    setStatus('loading');
    try {
      const mod = await import(
        /* webpackIgnore: true */ '/wasm/basic_js.js'
      );
      await mod.default();
      JsGridRef.current = mod.JsGrid;
      mountGrid(1_000);
      setStatus('ready');
    } catch (err) {
      console.error('WASM load failed:', err);
      setStatus('error');
    }
  }

  // Re-mount when rowCount changes
  useEffect(() => {
    if (status === 'ready') {
      mountGrid(rowCount);
    }
  }, [rowCount, status, mountGrid]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      gridRef.current?.detach();
      gridRef.current = null;
    };
  }, []);

  // Re-apply theme when dark/light mode toggles
  useEffect(() => {
    const observer = new MutationObserver(() => {
      gridRef.current?.set_theme_from_css();
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });
    return () => observer.disconnect();
  }, []);

  // ResizeObserver for responsive canvas
  useEffect(() => {
    const wrapper = containerRef.current?.querySelector(
      '[data-canvas-wrapper]',
    ) as HTMLElement | null;
    if (!wrapper) return;

    const ro = new ResizeObserver(() => {
      if (resizeTimer.current) clearTimeout(resizeTimer.current);
      resizeTimer.current = setTimeout(() => {
        if (status === 'ready') {
          mountGrid(rowCount);
        }
      }, 150);
    });
    ro.observe(wrapper);
    return () => ro.disconnect();
  }, [status, rowCount, mountGrid]);

  return (
    <div ref={containerRef} className={styles.demoWrapper}>
      <div className={styles.demoControls}>
        {ROW_OPTIONS.map((opt) => (
          <button
            key={opt.value}
            className={`${styles.rowBtn} ${rowCount === opt.value ? styles.rowBtnActive : ''}`}
            onClick={() => setRowCount(opt.value)}
          >
            {opt.label} {t('demo.rows')}
          </button>
        ))}
      </div>

      <div className={styles.canvasWrapper} data-canvas-wrapper>
        {status === 'loading' && (
          <div className={styles.loadingOverlay}>
            <div className={styles.spinner} />
            <span>{t('demo.loading')}</span>
          </div>
        )}
        {status === 'error' && (
          <div className={styles.errorFallback}>
            <p>{t('demo.error')}</p>
            <a href="/getting-started">{t('demo.errorLink')}</a>
          </div>
        )}
        <canvas
          ref={canvasRef}
          style={{
            width: '100%',
            height: '100%',
            display: status === 'error' ? 'none' : 'block',
          }}
        />
      </div>

      {status === 'ready' && (
        <p className={styles.demoHint}>{t('demo.hint')}</p>
      )}
    </div>
  );
}
