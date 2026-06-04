# rs-grid — Extraction des démos · état & reprise

_Dernière mise à jour : 2026-06-02._
Objectif : sortir les 4 démos de `rs-grid` vers des repos séparés (org `ruxelion`),
qui consomment la lib via **git tag**. Plan complet :
`C:\Users\Admin\.claude\plans\je-veux-separer-les-dapper-cocke.md`.

## ✅ Fait

**Repos d'exemples (sur GitHub, pushés, vérifiés) :**
- `ruxelion/rs-grid-example-{js,dioxus,yew,leptos}` — créés + pushés (par l'utilisateur).
- Locaux : `e:\Dev\ruxelion\rs-grid-example-*` (chacun = repo git, remote OK, `main` en sync).
- Deps `path` → `{ git = ".../rs-grid", tag = "v0.1.0" }`. Thèmes vendorés (`themes/`).
  Crate names gardés (`basic-js`… → wasm-pack sort `basic_js.*`, le site n'est pas cassé).
- Leptos : pipeline Tailwind + suite e2e migrée (specs + 26 snapshots win32).
- **Buildés OK contre `v0.1.0`** : dioxus/yew/leptos `trunk build` (6 thèmes dans dist),
  js `wasm-pack build` (`basic_js.{js,wasm}`). Chaque `Cargo.lock` (+leptos `package-lock.json`)
  committé + pushé.

**Tag (Phase A) :** `v0.1.0` poussé sur `ruxelion/rs-grid` → pointe `origin/main` (commit `0a2776b`).
Pas de `release.yml` sur main → aucun rebuild de site déclenché.

**rs-grid — branche `feat/extract-demos` (sur origin pour C, local pour D) :**
- `7564f62` (Phase C, pushé par l'utilisateur) : ajout `e2e/fixture-leptos` (app Leptos minimale
  = nouvelle cible e2e/CI/Pages) ; repoint playwright.config.ts, csp-server.js, ci.yml, csp.yml,
  pages.yml ; générateur class-map déplacé dans `tools/class-map/`.
- `3864243` (Phase D, **commit local, NON pushé, NON mergé**) : suppression des 4 démos +
  specs e2e orphelins (editing, leptos-component) ; nettoyage members/gitignore/CI/dependabot/
  launch.json/`/e2e`/Justfile/README/AGENTS/CONTRIBUTING/examples-README/docs.
- Vérifié : `cargo check --workspace` ✅ · `cargo nextest` **610/610** ✅ · `just --list` ✅ ·
  fixture : smoke+controls 7/7, CSP 2/2 ✅.

## ⏳ Reste à faire (tout est outward/irréversible → décision utilisateur)

### 1. Merger `feat/extract-demos` → `main` (rs-grid)
C'est ce qui fait réellement basculer `main` (perd les démos, CI/Pages sur le fixture).
```sh
cd e:\Dev\ruxelion\rs-grid
git push origin feat/extract-demos      # pousser la branche (Phase D incluse)
# puis : PR sur GitHub, OU fast-forward direct de main
```

### 2. Phase E — recâbler `ruxelion/rs-grid-site` (local : `e:\Dev\ruxelion\rs-grid-site`)
Le site build encore les démos depuis `rs-grid/examples/*` (supprimés). À faire :
- **`scripts/update-wasm-demo.sh`** : remplacer `RS_GRID="${1:-../../rs-grid}"` par une source par
  repo (`JS_SRC`/`LEPTOS_SRC`/`DIOXUS_SRC`/`YEW_SRC`, défaut `../../rs-grid-example-<fw>`).
  - bloc js : `cd "$JS_SRC"` (racine du repo, plus `/examples/...`).
  - `build_framework_demo` : sélectionner la source par framework (`case`).
  - reste (`trunk build --public-url /demos/$fw/`, `cp -r dist/.`) inchangé.
- **`.github/workflows/rebuild-demos.yml`** : remplacer le checkout unique de `rs-grid` par
  **4 checkouts** `ruxelion/rs-grid-example-<fw>` (chacun son `path`, `token: secrets.DEPLOY_TOKEN`).
  Garder le trigger `repository_dispatch: [rs-grid-released]`. Adapter l'appel du script +
  `cache-dependency-path` (`hashFiles('rs-grid-example-*/**/Cargo.lock')`) + Node cache
  (`rs-grid-example-leptos/package.json`). Garder `~/.cargo/git` en cache.
- Vérif : dispatch manuel de `rebuild-demos.yml` → `docs/public/wasm/basic_js*` +
  `docs/public/demos/{leptos,dioxus,yew}/` produits ; charger `/demos` + home `<GridDemo>`.

### (optionnel) `release.yml` sur rs-grid
Sur tag `v*` : build de sanity + `repository_dispatch` `rs-grid-released` (ref=tag) vers le site.
À ajouter sur la branche avant merge si on veut l'auto-rebuild aux futures releases.

## ⚠️ Gotchas / faits clés
- `rs-grid` est **privé** ; le site = repo **`ruxelion/rs-grid-site`** (public), local `rs-grid-site/`.
- `rs-grid/CLAUDE.md` est un **symlink → AGENTS.md** : éditer `AGENTS.md`.
- Pas de `site/` dans rs-grid (recettes Justfile `site`/`build-site` étaient mortes → supprimées).
- `tools/class-map` épinglé daisyui **5.5.19** (= version qui a généré `class_map_data.rs`) ;
  `just gen-class-map` reproduit les données à l'identique (le commit le diff = formatage rustfmt + date).
- Le fixture e2e couvre smoke+functional+CSP ; la régression **visuelle** vit dans le repo leptos
  (CI rs-grid fait `--grep-invert "visual regression|..."` + `--update-snapshots`).
- `gh` authentifié = **bpodwinski** (scopes repo + workflow).
- Limite connue : `e2e/tests/grid.spec.ts` (dans rs-grid) contient encore des tests screenshot
  « stylés » dormants en CI mais qui échoueraient sur un `npm test` local complet contre le fixture
  (à trimmer plus tard si on veut un `just e2e` local 100% vert).

## Reprise
Mémoire persistante : `demo-extraction-plan.md` (index `MEMORY.md`). Dire « on reprend
l'extraction des démos » et pointer ce fichier.
