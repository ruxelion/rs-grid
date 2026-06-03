// Minimal static server that serves the built dist/ with strict CSP headers.
// Used exclusively by the CSP test suite (playwright.csp.config.ts).
// Port 4174 to avoid conflict with the standard e2e server on 4173.
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(__dirname, 'fixture-leptos/dist');
const PORT = 4174;

// Strict CSP for a Trunk-built Leptos/WASM application.
// 'wasm-unsafe-eval' is required to instantiate WebAssembly modules.
// 'unsafe-inline' on style-src accommodates Leptos hydration markers —
// tighten to a hash once the build produces stable inline content.
const CSP = [
  "default-src 'none'",
  "script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline'",
  "style-src 'self' 'unsafe-inline'",
  // ui-avatars.com is an intentional external dependency used in demo data
  "img-src 'self' data: https://ui-avatars.com",
  "connect-src 'self'",
  "worker-src 'self' blob:",
  "font-src 'self'",
].join('; ');

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js':   'application/javascript',
  '.mjs':  'application/javascript',
  '.wasm': 'application/wasm',
  '.css':  'text/css',
  '.png':  'image/png',
  '.svg':  'image/svg+xml',
  '.ico':  'image/x-icon',
};

const server = http.createServer((req, res) => {
  let urlPath = req.url.split('?')[0];
  if (urlPath === '/' || urlPath === '') urlPath = '/index.html';

  const filePath = path.join(DIST, urlPath);
  const ext = path.extname(filePath);

  const serve = (p) => {
    try {
      const data = fs.readFileSync(p);
      res.writeHead(200, {
        'Content-Type': MIME[path.extname(p)] || 'application/octet-stream',
        'Content-Security-Policy': CSP,
        'X-Content-Type-Options': 'nosniff',
      });
      res.end(data);
      return true;
    } catch {
      return false;
    }
  };

  if (!serve(filePath)) {
    // SPA fallback: return index.html for unknown paths
    if (!serve(path.join(DIST, 'index.html'))) {
      res.writeHead(404);
      res.end('Not found');
    }
  }
});

server.listen(PORT, () => {
  // Playwright webServer waits for this exact URL to be reachable
  console.log(`CSP server listening on http://localhost:${PORT}`);
});
