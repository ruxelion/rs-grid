# Premiers pas

## Prerequis

- Toolchain Rust (stable, edition 2021)
- [Trunk](https://trunkrs.dev) pour compiler et servir les applications WASM
- La cible `wasm32-unknown-unknown`

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Ajouter la dependance

Ajoutez `rs-grid-leptos` a votre `Cargo.toml`. Tant que le crate n'est pas
encore publie sur crates.io, referencez-le par chemin :

```toml title="Cargo.toml"
[dependencies]
rs-grid-leptos = { path = "../rs-grid-leptos" }
```

## Utilisation de base

Importez le composant et montez-le dans une vue Leptos :

```rust title="src/main.rs"
use leptos::*;
use rs_grid_leptos::GridCanvas;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main style="width: 100vw; height: 100vh;">
            <GridCanvas
                rows=1_000_000_u64
                cols=50_usize
            />
        </main>
    }
}

fn main() {
    leptos::mount_to_body(App);
}
```

## Lancer en local

1. **Naviguez vers l'exemple**

   ```bash
   cd examples/basic-leptos
   ```

2. **Demarrez le serveur de developpement**

   ```bash
   trunk serve
   ```

3. **Ouvrez dans votre navigateur**

   Rendez-vous sur `http://localhost:8080`. Vous devriez voir un grid
   affichant 1 million de lignes a 60 fps.

## Compiler pour la production

```bash
cd examples/basic-leptos
trunk build --release
```

La sortie est generee dans `dist/`. Servez-la avec n'importe quel serveur de
fichiers statiques ou utilisez l'[image Docker](/fr/deployment/docker.md)
fournie.

## Commandes du workspace

| Commande                                  | Description                                        |
| ----------------------------------------- | -------------------------------------------------- |
| `cargo check --workspace`                 | Verification rapide des types pour tous les crates |
| `cargo test --workspace`                  | Lancer tous les tests unitaires                    |
| `cargo fmt --all`                         | Formater l'ensemble du workspace                   |
| `cargo clippy --workspace -- -D warnings` | Lint avec warnings traites comme erreurs           |
| `trunk serve` (dans le dossier exemple)   | Serveur de dev avec hot reload                     |

:::tip
`rs-grid-core` n'a aucune dependance WASM — ses tests unitaires s'executent
avec un simple `cargo test`, sans navigateur.
:::
