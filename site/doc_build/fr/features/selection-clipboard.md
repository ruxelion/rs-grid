# Sélection et presse-papiers

## Modèle de sélection

rs-grid utilise un modèle de sélection **ancre/focus** (comme les tableurs) :

- **Ancre** — la cellule où la sélection a commencé (clic)
- **Focus** — la cellule où la sélection se termine (Maj+clic ou touches fléchées)

La plage sélectionnée est le rectangle entre l'ancre et le focus.

Pour plus de détails sur le modèle de sélection de base, voir [Sélection](/fr/concepts/selection.md).

## Copier

```rust
let output = state.apply(GridCommand::CopySelection);
if let CommandOutput::CopyText(tsv) = output {
    // Écrire `tsv` dans le presse-papiers système
}
```

La sélection est sérialisée en **TSV** (valeurs séparées par des tabulations, RFC 4180) :

- Colonnes séparées par `\t`
- Lignes séparées par `\n`

### Copier avec les en-têtes

Disponible comme action intégrée du menu contextuel (`BuiltinAction::CopyWithHeaders`).
Ajoute une ligne d'en-tête en début de sortie TSV.

## Couper

```rust
let output = state.apply(GridCommand::CutSelection);
```

Identique à copier, mais efface également les cellules sélectionnées.
L'effacement est enregistré dans l'historique d'annulation.

## Coller

```rust
state.apply(GridCommand::PasteAt {
    text: "Alice\t30\nBob\t25".into(),
});
```

Colle le texte TSV à partir de l'ancre de sélection actuelle. Chaque valeur
séparée par une tabulation est écrite dans la cellule correspondante. Le collage
est enregistré comme une entrée d'annulation groupée.

## Raccourcis clavier

| Raccourci  | Action              |
| ---------- | ------------------- |
| **Ctrl+C** | Copier la sélection |
| **Ctrl+X** | Couper la sélection |
| **Ctrl+V** | Coller à l'ancre    |

Ces raccourcis sont gérés par la couche web (`rs-grid-web`) qui écoute les
événements DOM `copy`, `cut` et `paste`.

## Gestion des erreurs

```rust
pub enum CopyError {
    NoSelection,    // rien de sélectionné
    TooManyRows,    // la sélection dépasse MAX_COPY_ROWS
}
```

Si la sélection est trop grande, `CopySelection` renvoie
`CommandOutput::CopyError(CopyError::TooManyRows)` au lieu du texte TSV.

## Menu contextuel

Un clic droit ouvre un menu contextuel avec les actions du presse-papiers. Voir
[Menu contextuel](/fr/features/context-menu.md) pour les options de personnalisation.
