# Extraction des donnees

rs-grid n'inclut pas d'export CSV/JSON integre. A la place, `GridModel`
expose toutes les donnees necessaires pour construire votre propre export
en quelques lignes de code.

## Surface API cle

| Champ / Methode             | Ce qu'il fournit                                                         |
| --------------------------- | ------------------------------------------------------------------------ |
| `model.columns`             | `Vec<ColumnDef>` — definitions de colonnes ordonnees                     |
| `model.data.row_count()`    | Nombre total de lignes physiques                                         |
| `model.data.get_cell(r, k)` | Valeur de cellule par index physique et cle colonne                      |
| `model.data.get_cell_ref()` | Variante zero-copy `Cow<str>` (sources en memoire)                       |
| `model.sort_order`          | Indices physiques dans l'ordre de tri actuel (vide = ordre naturel)      |
| `model.filtered_indices`    | Indices physiques passant tous les filtres actifs (vide = pas de filtre) |
| `model.patches`             | Valeurs editees `(row, col_key) → value`                                 |

## Exemple de base : exporter toutes les lignes

```rust
let row_count = model.data.row_count();

for row in 0..row_count {
    for col in &model.columns {
        // Verifier les patches d'abord, puis la source de donnees
        let value = model
            .patches
            .get(&(row, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(row, &col.key))
            .unwrap_or_default();

        // Ecrire `value` dans votre sortie (CSV writer, tableau JSON, etc.)
    }
}
```

## Respecter le tri et le filtrage

Si vous voulez que l'export corresponde a ce que l'utilisateur voit dans la
grille, iterez sur `filtered_indices` (si actif) ou `sort_order` (si actif)
au lieu des indices bruts :

```rust
let indices: Vec<u64> = if !model.filtered_indices.is_empty() {
    // Les indices filtres sont deja dans l'ordre de tri
    model.filtered_indices.clone()
} else if !model.sort_order.is_empty() {
    model.sort_order.clone()
} else {
    (0..model.data.row_count()).collect()
};

for &phys in &indices {
    for col in &model.columns {
        let value = model
            .patches
            .get(&(phys, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(phys, &col.key))
            .unwrap_or_default();
        // ...
    }
}
```

## En-tetes de colonnes

Utilisez les champs de `ColumnDef` pour les libelles :

```rust
let headers: Vec<&str> = model
    .columns
    .iter()
    .map(|c| c.label.as_str())
    .collect();
```

## Sources de donnees server-side

Pour `PageCacheDataSource` ou les sources server-side personnalisees, seules
les lignes deja chargees dans le cache local sont disponibles. Verifiez
`cell_status()` avant de lire :

```rust
use rs_grid_core::datasource::CellStatus;

match model.data.cell_status(row, &col.key) {
    CellStatus::Ready(val) => { /* utiliser val */ }
    CellStatus::Loading    => { /* page pas encore chargee */ }
    CellStatus::Absent     => { /* pas de valeur */ }
}
```

Pour un export complet de donnees server-side, recupérez toutes les pages
directement depuis votre backend plutot que de lire a travers la grille.

## Persister les modifications

`GridCanvas` expose deux methodes pour sauvegarder et restaurer la couche
de patches (l'ensemble des cellules editees par l'utilisateur) :

| Methode                      | Description                                                     |
| ---------------------------- | --------------------------------------------------------------- |
| `export_patches() -> String` | Serialise toutes les cellules editees en TSV versionne          |
| `import_patches(data: &str)` | Deserialise et applique les patches en remplacant les existants |

### Format

La chaine exportee commence par un en-tete de version, suivi d'une ligne
par cellule editee :

```
rs-grid-patches/v1
0	salary	42000
3	name	Alice
7	country	FR
```

Chaque ligne de donnees est `ligne_physique\tcle_colonne\tvaleur`. Les
caracteres tabulation, saut de ligne et antislash dans les cles et valeurs
sont echappes en `\t`, `\n`, `\\`. L'en-tete de version permet aux
versions futures de migrer les donnees sauvegardees sans les corrompre
silencieusement.

:::note
`import_patches` accepte egalement les donnees sans en-tete de version
(produites avant l'introduction de ce format), afin que les patches
precedemment sauvegardes restent chargeables apres une mise a jour.
:::

### Exemple localStorage (Leptos)

```rust
const LS_KEY: &str = "my-grid-patches";

// Sauvegarder a chaque edition
let gc2 = gc.clone();
gc.set_on_change(move || {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(LS_KEY, &gc2.export_patches());
    }
});

// Restaurer au montage
if let Some(storage) = local_storage() {
    if let Ok(Some(data)) = storage.get_item(LS_KEY) {
        gc.import_patches(&data);
    }
}
```

### Telechargement / upload de fichier (Vanilla JS)

```js
// Export → telechargement
const tsv = grid.export_patches();
const url = URL.createObjectURL(
  new Blob([tsv], { type: "text/tab-separated-values" })
);
Object.assign(document.createElement("a"), {
  href: url, download: "patches.tsv"
}).click();
URL.revokeObjectURL(url);

// Import ← input fichier
fileInput.addEventListener("change", async (e) => {
  const text = await e.target.files[0].text();
  grid.import_patches(text);
});
```

### Quand utiliser les patches vs une sauvegarde complete

| Scenario                                 | Recommandation                                                     |
| ---------------------------------------- | ------------------------------------------------------------------ |
| Petites editions sur un dataset statique | `export_patches` — compact, rapide                                 |
| Persistance complete du dataset          | Iterer `model.patches` + votre propre serialisation                |
| Source de donnees server-side            | Envoyer les editions individuelles a votre API via `set_on_change` |
