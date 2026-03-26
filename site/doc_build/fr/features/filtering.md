# Filtrage

## Vue d'ensemble

rs-grid prend en charge le filtrage textuel par colonne. Chaque filtre effectue
une recherche de type « contient » insensible à la casse sur la valeur de la cellule.

## Commandes

### Définir un filtre

```rust
state.apply(GridCommand::SetColumnFilter {
    col_key: "name".into(),
    text: "john".into(),
});
```

Passez une chaîne vide pour supprimer le filtre de cette colonne :

```rust
state.apply(GridCommand::SetColumnFilter {
    col_key: "name".into(),
    text: "".into(),
});
```

### Effacer tous les filtres

```rust
state.apply(GridCommand::ClearAllFilters);
```

## Fonctionnement

### Mode côté client (par défaut)

Lorsqu'un filtre est défini, `apply_filter()` parcourt toutes les lignes et
construit `filtered_indices: Vec<u64>` — la liste des indices de lignes
physiques qui satisfont tous les filtres actifs, stockée dans l'ordre de tri.

Les filtres multiples sont combinés par ET : une ligne doit correspondre à
tous les filtres de colonnes actifs pour être visible.

`model.display_row_count()` renvoie le nombre de lignes filtrées (ou le
total lorsqu'aucun filtre n'est actif).

:::warning
Le filtrage côté client est conçu pour des jeux de données allant jusqu'à \~1 million de lignes.
Pour des volumes plus importants, utilisez le mode côté serveur.
:::

### Mode côté serveur

Lorsque `model.mode = DataSourceMode::ServerSide`, `apply_filter()` est un
no-op. L'état du filtre est néanmoins stocké dans `model.filters` pour que
votre application puisse le lire et le transmettre au serveur.

## État des filtres

Les filtres actifs sont stockés dans `model.filters: HashMap<String, String>`,
associant les clés de colonnes au texte de filtre. Vous pouvez lire cette
structure pour construire des requêtes serveur :

```rust
for (col_key, text) in &state.model.filters {
    println!("Filtre sur {}: {}", col_key, text);
}
```

## Interaction avec le tri

Le filtrage respecte l'ordre de tri actif. Lorsque les deux sont actifs,
`filtered_indices` contient les indices de lignes physiques dans l'ordre trié.
