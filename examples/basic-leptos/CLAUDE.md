# examples/basic-leptos

Application de démonstration de rs-grid. Construite avec Leptos 0.7 CSR + Trunk.

## Build et dev

```sh
# Serveur de développement (hot-reload)
trunk serve

# Build de production (requis avant les tests Playwright)
trunk build

# Build release (WASM optimisé)
trunk build --release
```

L'app est servie sur `http://localhost:8080` par défaut (`trunk serve`).
Les tests Playwright utilisent le `dist/` pré-compilé servi sur le port 4173.

## Structure

```
basic-leptos/
├── src/lib.rs            # Composant App + build_model
├── index.html            # Template Trunk (directives data-trunk)
├── Trunk.toml            # Config build, chemins de watch
├── rs-grid-theme.css     # Variables CSS du thème + styles app
└── dist/                 # Sortie compilée (ignorée par git)
```

## Données virtuelles (`src/lib.rs`)

`build_model(row_count, col_count)` crée un `GridModel` backed par un
`FnDataSource` — les données sont générées à la volée, rien n'est stocké en
mémoire. Les colonnes disponibles sont :

| Clé | Exemple |
|---|---|
| `id` | `0`, `1`, `2`... |
| `name` | `User 0`, `User 1`... |
| `email` | `user0@example.com`... |
| `role` | `Admin` (row % 3 == 0) ou `Member` |
| `dept` | `Dept 0`..`Dept 19` (row % 20) |
| `status` | `Inactive` (row % 5 == 0) ou `Active` |
| `colN` | `{row}×{N}` pour les colonnes extra |

Pour tester avec des données réelles (VecDataSource), utiliser `GridModel::new`
à la place de `GridModel::with_data_source`.

## Thème CSS (`rs-grid-theme.css`)

Toutes les variables `--rs-grid-*` sont définies sur `:root`. Ce fichier fait
deux choses :
1. Variables de thème pour la grille (lues par `theme_from_css_vars()`)
2. Styles de l'app elle-même (`.app-layout`, `.app-controls`, etc.)

Pour tester le thème sombre, décommenter le bloc `:root { ... }` en bas du
fichier. Ne pas modifier les noms des variables — ils sont lus par
`rs-grid-web::css_theme`.

## Trunk.toml

Le `[watch]` surveille tous les crates du workspace. Trunk rebuilde
automatiquement si on touche à n'importe quelle crate source.
`dist = "dist"` et `release = false` par défaut — ne pas committer le `dist/`.

## Invariants

- **CSR uniquement** — pas de SSR, pas de `#[server]`. Tout le code s'exécute
  dans le navigateur.
- `build_model` doit retourner un `GridModel` frais à chaque appel — Leptos
  recrée le composant `<GridCanvas>` quand `row_count` ou `col_count` change.
- Ne pas passer `GridModel` via un signal Leptos — il n'est pas `Clone`.
  Le reconstruire dans le closure réactif (pattern déjà en place).
