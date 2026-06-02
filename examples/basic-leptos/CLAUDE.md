# examples/basic-leptos

Application de démonstration de rs-grid. Construite avec Leptos 0.8 CSR + Trunk.

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

## Données virtuelles (`src/lib.rs` + `src/fake_data.rs`)

`build_model(row_count, col_count)` crée un `GridModel` backed par un
`FnDataSource` — les données sont générées à la volée via `fake_data::fake_cell()`,
rien n'est stocké en mémoire. Un hash déterministe (splitmix64) assure que chaque
ligne produit toujours les mêmes valeurs.

| Clé | Exemple | Mécanisme |
|---|---|---|
| `id` | `0`, `1`, `2`... | row index |
| `name` | `Alice Johnson`, `Miguel Torres`... | ~100 prénoms × ~120 noms, hash decorrelé |
| `email` | `alice.johnson@example.com`... | dérivé du nom |
| `role` | `Senior Software Engineer`, `CTO`... | ~20 titres avec fourchettes de salaire |
| `dept` | `Engineering`, `Marketing`... | 12 départements |
| `salary` | `142000`, `65000`... | fourchette liée au rôle |
| `active` | `true`, `false` | ~85% actifs |
| `avatar` | `data:image/svg+xml;base64,…` | initiales prénom+nom → SVG data-URI local (aucun appel réseau) |
| `colN` | `{row}x{N}` | colonnes extra (mode 100 colonnes) |

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
