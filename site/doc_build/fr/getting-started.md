# Premiers pas

## Prerequis

- Toolchain Rust (stable, edition 2021)
- [Trunk](https://trunkrs.dev) pour compiler et servir les applications WASM
- La cible `wasm32-unknown-unknown`

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Demarrage rapide


**[object Object]**

Ajoutez `rs-grid-leptos` a votre `Cargo.toml` :
```toml title="Cargo.toml"
[dependencies]
rs-grid-leptos = { path = "../rs-grid-leptos" }
```
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
Lancez l'exemple :
```bash
cd examples/basic-leptos
trunk serve
```


**[object Object]**

Compilez le package WASM :
```bash
cd crates/rs-grid-web
wasm-pack build --target web
```
Utilisez-le dans votre HTML :
```html title="index.html"
<canvas id="grid" style="width: 100vw; height: 100vh;"></canvas>
<script type="module">
  import init, { JsGrid } from './pkg/rs_grid_web.js';
  await init();
  const canvas = document.getElementById('grid');
  const grid = new JsGrid(canvas, 1_000_000, 50);
</script>
```


**[object Object]**

Ajoutez `rs-grid-dioxus` a votre `Cargo.toml` :
```toml title="Cargo.toml"
[dependencies]
rs-grid-dioxus = { path = "../rs-grid-dioxus" }
```
Importez le composant et montez-le dans une application Dioxus :
```rust title="src/main.rs"
use dioxus::prelude::*;
use rs_grid_dioxus::{GridCanvas, ModelSlot};
use rs_grid_core::model::GridModel;

fn App() -> Element {
    let model = use_hook(|| {
        ModelSlot::new(GridModel::new(1_000_000, 50))
    });
    rsx! {
        main { style: "width:100vw;height:100vh;",
            GridCanvas { model: model.clone() }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
```
Lancez l'exemple :
```bash
cd examples/basic-dioxus
trunk serve
```


**[object Object]**

Ajoutez `rs-grid-yew` a votre `Cargo.toml` :
```toml title="Cargo.toml"
[dependencies]
rs-grid-yew = { path = "../rs-grid-yew" }
```
Importez le composant et montez-le dans une application Yew :
```rust title="src/main.rs"
use yew::prelude::*;
use rs_grid_yew::{GridCanvas, wrap_model};
use rs_grid_core::model::GridModel;

#[function_component]
fn App() -> Html {
    let model = wrap_model(GridModel::new(1_000_000, 50));
    html! {
        <main style="width:100vw;height:100vh;">
            <GridCanvas model={model} />
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```
Lancez l'exemple :
```bash
cd examples/basic-yew
trunk serve
```


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
