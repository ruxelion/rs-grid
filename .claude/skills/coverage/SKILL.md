---
name: coverage
description: Run unit test coverage analysis with cargo-llvm-cov. Use when the user wants to check test coverage, find untested code, or generate coverage reports.
user-invocable: true
allowed-tools: Bash, Read, Grep, Glob, Agent
argument-hint: "[crate-name] [--html] [--uncovered]"
---

# Coverage Skill

Analyse la couverture de tests unitaires avec `cargo-llvm-cov` + `cargo-nextest`.

Parse `$ARGUMENTS` pour determiner le mode d'execution :

## Modes

1. **Sans argument** (`/coverage`) : couverture des crates natifs (core + scene + icons)
2. **Avec un nom de crate** (`/coverage <crate>`) : couverture d'un crate specifique (ex: `/coverage rs-grid-core`)
3. **Flag `--html`** (`/coverage --html` ou `/coverage <crate> --html`) : genere le rapport HTML et l'ouvre dans le navigateur
4. **Flag `--uncovered`** (`/coverage --uncovered` ou `/coverage <crate> --uncovered`) : analyse les zones faiblement couvertes et suggere des tests a ecrire

## Crates couverts par defaut

Seuls les crates compilables en cible native sont couverts :
- `rs-grid-core` — logique headless, 281 tests
- `rs-grid-scene` — conversion SceneFrame, 89 tests
- `rs-grid-icons` — assets SVG/icones

Les crates WASM (`rs-grid-web`, `rs-grid-leptos`, `rs-grid-dioxus`, `rs-grid-yew`,
`rs-grid-render-canvas`, exemples) sont exclus car ils ne se compilent pas en natif.

## Execution

**Etape 1 — Nettoyer les donnees de couverture precedentes**

```bash
cargo llvm-cov clean --workspace
```

**Etape 2 — Lancer la couverture**

Selon le mode detecte dans `$ARGUMENTS` :

- **Defaut (pas de crate specifie)** :
  ```bash
  cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --text
  ```

- **Crate specifique** (remplacer `<crate>` par le nom dans `$ARGUMENTS`) :
  ```bash
  cargo llvm-cov nextest -p <crate> --text
  ```

- **HTML (defaut)** :
  ```bash
  cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --html --open
  ```

- **HTML (crate specifique)** :
  ```bash
  cargo llvm-cov nextest -p <crate> --html --open
  ```

- **lcov (CI)** :
  ```bash
  cargo llvm-cov nextest -p rs-grid-core -p rs-grid-scene -p rs-grid-icons --lcov --output-path target/llvm-cov/lcov.info
  ```

Le timeout pour ces commandes doit etre de 10 minutes (600000ms).

**Etape 3 — Afficher les resultats**

- **Mode `--text`** : analyser la sortie et presenter un resume sous forme de tableau
  markdown avec les colonnes : Crate, Lignes couvertes, Lignes totales, Couverture (%).
  Mettre en evidence les crates avec une couverture < 50%.

- **Mode `--html`** : le flag `--open` ouvre automatiquement le navigateur.
  Indiquer a l'utilisateur que le rapport est disponible dans `target/llvm-cov/html/`.

**Etape 4 — Mode `--uncovered` (analyse approfondie)**

Si le flag `--uncovered` est present dans `$ARGUMENTS` :

1. Identifier les fichiers avec la couverture la plus faible dans la sortie
2. Lire les fichiers sources concernes (les 3 pires)
3. Identifier les fonctions et branches non couvertes
4. Proposer des tests unitaires concrets a ecrire, en suivant les patterns du projet :
   - Modules `#[cfg(test)] mod tests` inline dans le fichier source
   - Pas de mocks externes — les structs sont construites directement
   - Utiliser `GridState::apply(GridCommand)` pour tester les mutations d'etat
5. Presenter les suggestions sous forme de liste priorisee par impact

## Conventions du projet a respecter

- Tests inline `#[cfg(test)] mod tests { ... }` dans chaque fichier source
- Pas de `unwrap()` — utiliser `expect("raison")` ou propagation d'erreur
- Format rustfmt : `max_width=80`
- Commentaires et documentation en anglais US
- Toute nouvelle valeur visuelle doit passer par le systeme de theme (`Theme`, `light()`, `dark()`, `dimmed()`)
