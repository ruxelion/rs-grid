---
name: mintlify
description: |
  Build and maintain documentation sites with Mintlify. Use when creating docs
  pages, configuring navigation, adding MDX components, setting up API
  references, deploying to a custom domain, or making documentation agent-ready
  (MCP, skill.md, llms.txt).
license: MIT
metadata:
  version: 1.0.0
  category: documentation
  author: Claude Code Skills
  triggers:
    - create mintlify site
    - deploy mintlify
    - mintlify docs
    - add mintlify page
    - configure mintlify navigation
    - mintlify openapi
    - mintlify mcp
    - mintlify skill.md
---

# Mintlify Documentation

Build beautiful, AI-ready documentation sites from MDX files hosted on GitHub
or GitLab. Mintlify handles hosting, search, theming, and agent integrations.

## Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [docs.json Configuration](#docsjson-configuration)
- [Page Authoring](#page-authoring)
- [MDX Components](#mdx-components)
- [API Documentation](#api-documentation)
- [Customization](#customization)
- [AI & Agent Features](#ai--agent-features)
- [Deployment](#deployment)
- [Troubleshooting](#troubleshooting)

## Quick Start

```bash
# Install the Mintlify CLI
npm install -g mintlify

# Preview docs locally (run from the folder containing docs.json)
mintlify dev

# Check for broken links
mintlify broken-links
```

Connect your repository at [dashboard.mintlify.com](https://dashboard.mintlify.com).
Mintlify auto-deploys on every push to your default branch.

## Project Structure

```
docs/                   # or any folder — points to docs.json location
  docs.json             # navigation, theming, and site config (REQUIRED)
  index.mdx             # landing/home page
  quickstart.mdx
  api-reference/
    introduction.mdx
    endpoint/
      get-user.mdx
  _snippets/            # reusable content fragments
    common-warning.mdx
  images/               # static assets
  openapi.yaml          # optional OpenAPI spec
```

## docs.json Configuration

`docs.json` is the single source of truth for site structure.

### Minimal config

```json
{
  "name": "My Product",
  "navigation": [
    {
      "group": "Getting Started",
      "pages": ["index", "quickstart"]
    }
  ]
}
```

### Full config with theming

```json
{
  "name": "My Product",
  "logo": {
    "light": "/logo/light.svg",
    "dark": "/logo/dark.svg"
  },
  "favicon": "/favicon.svg",
  "colors": {
    "primary": "#3b82f6",
    "light": "#60a5fa",
    "dark": "#1d4ed8"
  },
  "topbarLinks": [
    { "name": "GitHub", "url": "https://github.com/org/repo" }
  ],
  "topbarCtaButton": {
    "name": "Get Started",
    "url": "https://app.example.com"
  },
  "navigation": [
    {
      "group": "Overview",
      "pages": ["index", "quickstart"]
    },
    {
      "group": "Guides",
      "pages": [
        "guides/installation",
        {
          "group": "Advanced",
          "pages": ["guides/advanced/caching", "guides/advanced/scaling"]
        }
      ]
    },
    {
      "group": "API Reference",
      "pages": ["api-reference/introduction", "api-reference/endpoint/get-user"]
    }
  ],
  "footerSocials": {
    "github": "https://github.com/org",
    "twitter": "https://twitter.com/org"
  }
}
```

### Navigation tips

- Page paths in `navigation` are **relative to docs.json** and **without `.mdx`**
- Nested groups: use an object `{ "group": "...", "pages": [...] }`
- Hidden pages (not in nav): create the `.mdx` file but don't add to `navigation`
- Tabs (top-level sections): use the `"tabs"` key instead of `"navigation"`

```json
{
  "tabs": [
    {
      "name": "Documentation",
      "url": "docs",
      "navigation": [...]
    },
    {
      "name": "API Reference",
      "url": "api-reference",
      "navigation": [...]
    }
  ]
}
```

## Page Authoring

### Frontmatter

Every `.mdx` page should have frontmatter:

```mdx
---
title: "Page Title"
description: "One-line description shown in search results and meta tags."
icon: "rocket"
---
```

| Field | Required | Description |
|-------|----------|-------------|
| `title` | Yes | Page title (H1) |
| `description` | Recommended | Used in SEO and search |
| `icon` | No | [Lucide](https://lucide.dev) or Font Awesome icon name |
| `sidebarTitle` | No | Shorter title for the sidebar |
| `mode` | No | `"wide"` removes the right sidebar |
| `noindex` | No | `true` excludes from search index |

### File naming

- Use **kebab-case**: `getting-started.mdx`, `api-reference.mdx`
- Folder and file names become URL slugs automatically
- `index.mdx` in a folder maps to the folder URL (`/guides/` not `/guides/index`)

### Links

```mdx
<!-- Relative (preferred) -->
[Next step](/guides/next-step)

<!-- External -->
[GitHub](https://github.com)

<!-- Page anchor -->
[Installation](#installation)
```

## MDX Components

All components are available globally — no imports needed.

### Layout & Structure

```mdx
<CardGroup cols={2}>
  <Card title="Quickstart" icon="rocket" href="/quickstart">
    Get up and running in 5 minutes.
  </Card>
  <Card title="API Reference" icon="code" href="/api-reference">
    Full API documentation.
  </Card>
</CardGroup>
```

```mdx
<Steps>
  <Step title="Install">
    Run `npm install my-package`
  </Step>
  <Step title="Configure">
    Create a config file.
  </Step>
  <Step title="Deploy">
    Run `npm run deploy`
  </Step>
</Steps>
```

```mdx
<Tabs>
  <Tab title="npm">
    ```bash
    npm install my-package
    ```
  </Tab>
  <Tab title="yarn">
    ```bash
    yarn add my-package
    ```
  </Tab>
</Tabs>
```

### Callouts

```mdx
<Note>Informational note.</Note>
<Warning>This action is irreversible.</Warning>
<Tip>Pro tip for advanced users.</Tip>
<Info>Neutral information.</Info>
<Check>Success confirmation.</Check>
```

### Code

````mdx
```typescript title="example.ts"
const result = await fetch('/api/data');
```

```bash
npm install
```
````

With line highlighting:

````mdx
```python {2,4}
def greet(name):
    message = f"Hello, {name}"  # highlighted
    print(message)
    return message  # highlighted
```
````

### Accordion (FAQ)

```mdx
<AccordionGroup>
  <Accordion title="What is this?">
    An explanation of what this is.
  </Accordion>
  <Accordion title="How does it work?">
    Step by step explanation.
  </Accordion>
</AccordionGroup>
```

### Reusable snippets

Create `_snippets/my-warning.mdx`:

```mdx
<Warning>
  Always back up your data before proceeding.
</Warning>
```

Use it in any page:

```mdx
<Snippet file="my-warning.mdx" />
```

### Frames (images with border)

```mdx
<Frame>
  <img src="/images/screenshot.png" alt="Dashboard screenshot" />
</Frame>
```

## API Documentation

### OpenAPI (recommended)

1. Add your OpenAPI spec to the project:

```
docs/
  openapi.yaml   # or openapi.json
```

2. Reference it in `docs.json`:

```json
{
  "openapi": "openapi.yaml"
}
```

3. Auto-generate pages from the spec:

```bash
npx @mintlify/scraping@latest openapi-file openapi.yaml -o api-reference
```

This creates one `.mdx` file per endpoint.

4. Each generated page looks like:

```mdx
---
title: "Get User"
openapi: "GET /users/{id}"
---
```

### Manual API pages

Without OpenAPI, document endpoints manually using `ParamField`:

```mdx
---
title: "Get User"
description: "Retrieve a user by ID."
---

<ParamField path="id" type="string" required>
  The unique identifier of the user.
</ParamField>

<ResponseField name="id" type="string">
  The user's unique identifier.
</ResponseField>

<ResponseField name="email" type="string">
  The user's email address.
</ResponseField>
```

## Customization

### Custom fonts (Google Fonts)

```json
{
  "font": {
    "headings": { "family": "Inter" },
    "body": { "family": "Inter" }
  }
}
```

### Custom CSS

```json
{
  "custom": {
    "css": "/custom.css"
  }
}
```

`docs/custom.css`:

```css
:root {
  --mint-code-bg: #0d1117;
}
```

### Custom domain

1. Add a CNAME record pointing to `hosting.mintlify.com`
2. Set in `docs.json`:

```json
{
  "name": "My Product",
  "custom_domain": "docs.myproduct.com"
}
```

### Versioning (multiple doc sets)

```json
{
  "versions": ["v1", "v2"],
  "navigation": {
    "v2": [...],
    "v1": [...]
  }
}
```

## AI & Agent Features

### MCP server

Mintlify auto-generates an MCP server at `/mcp` for every public site.

Connect to Claude Code:

```bash
claude mcp add --transport http <site-name> https://your-docs-domain.com/mcp
```

The MCP exposes a search tool that queries your indexed documentation.

### llms.txt

Auto-generated at `/llms.txt` — lists all documentation pages with
descriptions for AI agents to discover content.

Fetch the full content:

```bash
curl https://your-docs-domain.com/llms-full.txt
```

### skill.md

Auto-generated at `/skill.md` following the
[agentskills.io specification](https://agentskills.io/specification).

Override with a custom file at the root of your project:

```md
---
name: my-product
description: Do X with my product. Use when Y.
license: MIT
compatibility: Requires Node.js 18+.
metadata:
  author: myorg
  version: "1.0"
---

# My Product

## Capabilities
...
```

Discovery endpoints:

- `/.well-known/agent-skills/index.json` — agent-skills 0.2.0 spec
- `/.well-known/skills/index.json` — classic skills index
- `/.well-known/skills/{name}/skill.md` — individual skill file

### Contextual AI menu

Add AI action buttons to every page via `docs.json`:

```json
{
  "contextualLinks": [
    { "type": "mcp", "label": "Copy MCP URL" },
    { "type": "add-mcp", "label": "Install MCP" },
    { "type": "cursor", "label": "Open in Cursor" },
    { "type": "vscode", "label": "Open in VS Code" }
  ]
}
```

## Deployment

### GitHub (recommended)

1. Connect repository at [dashboard.mintlify.com](https://dashboard.mintlify.com)
2. Set the docs folder path in the dashboard
3. Every push to the default branch triggers a deployment
4. PRs generate preview deployments automatically

### GitLab

Same flow — connect via dashboard, set the folder path.

### Monorepo

Set the docs folder explicitly in `docs.json`:

```json
{
  "name": "My Product",
  "basePath": "/docs"
}
```

Or configure in the dashboard under **Deployment → Docs folder path**.

### Preview locally

```bash
# Must be run from the directory containing docs.json
mintlify dev

# Specify a custom port
mintlify dev --port 3333
```

### CI/CD integration

```yaml
# .github/workflows/docs.yml
name: Check Docs
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: npm install -g mintlify
      - run: mintlify broken-links
```

## Troubleshooting

**`mintlify dev` shows blank page**
- Ensure `docs.json` is in the current directory
- Check that referenced page paths in `navigation` exist as `.mdx` files

**Page not appearing in sidebar**
- Verify the path is listed in `docs.json` `navigation`
- Path must be without the `.mdx` extension
- Check for typos: paths are case-sensitive

**OpenAPI page shows no content**
- Confirm `openapi` field in `docs.json` points to the correct file
- Validate the spec with [Swagger Editor](https://editor.swagger.io)
- Use `openapi: "METHOD /path"` in the page frontmatter, not the operationId

**Images not loading**
- Use absolute paths from the docs root: `/images/screenshot.png`
- Supported formats: PNG, JPG, SVG, GIF, WebP

**Broken links in CI**
```bash
mintlify broken-links --path docs
```

## Key Resources

- [Mintlify Documentation](https://mintlify.com/docs)
- [docs.json reference](https://mintlify.com/docs/settings/global)
- [Component library](https://mintlify.com/docs/content/components)
- [OpenAPI setup](https://mintlify.com/docs/api-playground/openapi-setup)
- [MCP server](https://mintlify.com/docs/ai/mcp)
- [skill.md spec](https://agentskills.io/specification)
- [Dashboard](https://dashboard.mintlify.com)
