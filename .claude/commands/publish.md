# /publish — Release checklist (crates.io)

Commande de release structurée. À chaque étape marquée **[STOP]**, présenter
l'action à effectuer et attendre la confirmation explicite de l'utilisateur avant
de continuer. Ne jamais exécuter une étape de manière autonome.

## Répartition des rôles

Depuis l'intégration de **release-plz** (mode release-PR-only) :

- **release-plz** (automatique, CI) : ouvre une PR de release qui **bumpe les
  versions** (indépendantes par crate) et **génère les `crates/*/CHANGELOG.md`**
  depuis les commits conventionnels.
- **Humain** : review + merge de la PR de release sur `main`.
- **`/publish`** (cette commande, manuel) : **publie** les crates modifiés sur
  crates.io dans l'ordre de dépendances, puis **crée et pousse les tags per-crate**.

`/publish` ne bumpe plus les versions et n'édite plus le changelog — c'est le
travail de la PR de release-plz, déjà mergée à ce stade.

> Évolution future (« Phase B ») : activer le job `release` de release-plz
> (publication + tags + GitHub releases automatiques) rendra cette commande
> obsolète. Voir `.github/workflows/release-plz.yml`.

## Pré-conditions

- La PR de release-plz est **mergée sur `main`** (versions + changelogs à jour).
- Le working tree est propre et à jour sur `main`.
- Noter la liste **crate → nouvelle version** depuis la description de la PR de
  release-plz (seuls les crates listés ont changé et sont à publier/tagger).

---

## Étapes

### 1. CI locale [STOP]

```sh
just ci        # fmt + lint + tests (natif, WASM crates exclus)
just e2e       # trunk build + Playwright
```

Les deux doivent passer en vert avant de continuer.

### 2. Publication crates.io (ordre obligatoire) [STOP]

Publier **uniquement les crates modifiés** par la PR de release-plz, dans cet
ordre exact (respect du graphe de dépendances). Attendre ~30 secondes entre
chaque publication pour que crates.io indexe le crate.

```sh
cargo publish -p rs-grid-core
# attendre 30s
cargo publish -p rs-grid-icons
# attendre 30s
cargo publish -p rs-grid-scene
# attendre 30s
cargo publish -p rs-grid-render-canvas
# attendre 30s
cargo publish -p rs-grid-web
# attendre 30s
cargo publish -p rs-grid-leptos
# attendre 30s
cargo publish -p rs-grid-dioxus
# attendre 30s
cargo publish -p rs-grid-yew
```

En cas d'erreur 429 (rate limit), attendre le délai indiqué dans le message
d'erreur avant de relancer le crate bloqué.

### 3. Vérification [STOP]

```sh
cargo search rs-grid-core
```

Confirmer que la nouvelle version de chaque crate publié apparaît.

### 4. Tags per-crate + push [STOP]

Créer un tag par crate publié, au format `rs-grid-<crate>-v<version>` (les
versions viennent de la PR de release-plz). Exemple si seuls core et scene ont
changé :

```sh
git tag rs-grid-core-v0.2.0
git tag rs-grid-scene-v0.1.3
# … un tag par crate publié

git push origin main
git push origin --tags
```

Le push des tags déclenche `release.yml` (sanity build + notification du site
`rs-grid-site`). Tous les tags pointant sur le même commit, la concurrency
`site-notify` collapse les notifications en un seul rebuild.

---

## En cas de problème

- **"no matching package"** : le crate upstream n'est pas encore indexé — attendre
  30–60s et réessayer.
- **400 email not verified** : vérifier l'email sur crates.io/settings/profile.
- **"crate already exists"** : le crate a déjà été publié à cette version — ne pas
  relancer, passer à la suite.
