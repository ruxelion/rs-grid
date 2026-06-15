# /publish — Release checklist (crates.io)

Commande de release structurée. À chaque étape marquée **[STOP]**, présenter
l'action à effectuer et attendre la confirmation explicite de l'utilisateur avant
de continuer. Ne jamais exécuter une étape de manière autonome.

## Pré-conditions

- Tous les changements sont commités sur `main`
- La version cible est confirmée (patch / minor — voir section Versioning dans CLAUDE.md)

---

## Étapes

### 1. Bump de version [STOP]

Modifier `version` dans `[workspace.package]` du `Cargo.toml` racine :

```toml
[workspace.package]
version = "X.Y.Z"   # ← nouvelle version
```

Vérifier qu'aucun `Cargo.lock` ne réclame l'ancienne version :
```sh
cargo check --workspace
```

### 2. CI locale [STOP]

```sh
just ci        # fmt + lint + tests (natif, WASM crates exclus)
just e2e       # trunk build + Playwright
```

Les deux doivent passer en vert avant de continuer.

### 3. Mettre à jour CHANGELOG.md [STOP]

- Renommer la section `[Unreleased]` en `[X.Y.Z] — YYYY-MM-DD`
- Ajouter une nouvelle section `[Unreleased]` vide en haut

### 4. Commit de release [STOP]

```sh
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "release: vX.Y.Z"
```

### 5. Tag git [STOP]

```sh
git tag vX.Y.Z
```

Ne pas pousser encore — attendre que la publication crates.io réussisse.

### 6. Publication crates.io (ordre obligatoire) [STOP]

Publier dans cet ordre exact (respect du graphe de dépendances).
Attendre ~30 secondes entre chaque publication pour que crates.io indexe le crate.

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

### 7. Vérification [STOP]

```sh
cargo search rs-grid-core
```

Confirmer que la nouvelle version apparaît dans les résultats.

### 8. Push [STOP]

```sh
git push origin main
git push origin vX.Y.Z
```

---

## En cas de problème

- **"no matching package"** : le crate upstream n'est pas encore indexé — attendre
  30–60s et réessayer.
- **400 email not verified** : vérifier l'email sur crates.io/settings/profile.
- **"crate already exists"** : le crate a déjà été publié à cette version — ne pas
  relancer, passer à la suite.
