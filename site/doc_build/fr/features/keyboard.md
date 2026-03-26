# Navigation clavier

## Modele de focus

Le canvas de la grille doit avoir le focus DOM pour recevoir les evenements
clavier. Cliquez sur le canvas ou utilisez la navigation par tabulation pour
lui donner le focus.

## Raccourcis de navigation

| Raccourci                   | Action                                         |
| --------------------------- | ---------------------------------------------- |
| **Touches directionnelles** | Deplacer la selection d'une cellule            |
| **Shift + Fleche**          | Etendre la selection dans cette direction      |
| **Entree**                  | Commencer l'edition de la cellule selectionnee |
| **Echap**                   | Annuler l'edition ou effacer la selection      |
| **Ctrl+Z**                  | Annuler                                        |
| **Ctrl+Y**                  | Retablir                                       |
| **Ctrl+C**                  | Copier la selection                            |
| **Ctrl+X**                  | Couper la selection                            |
| **Ctrl+V**                  | Coller a l'ancre de selection                  |
| **Ctrl+F**                  | Ouvrir la barre de recherche                   |

## Commande MoveSelection

Les appuis sur les touches directionnelles se traduisent en `MoveSelection` :

```rust
state.apply(GridCommand::MoveSelection {
    delta_row: -1,  // vers le haut
    delta_col: 0,
    extend: false,  // true si Shift est maintenu
});
```

| Touche | delta\_row | delta\_col |
| ------ | ---------- | ---------- |
| Haut   | -1         | 0          |
| Bas    | +1         | 0          |
| Gauche | 0          | -1         |
| Droite | 0          | +1         |

Lorsque `extend` vaut `true`, l'ancre reste fixe et seul le focus se deplace,
ce qui etend la plage selectionnee.

## Edition au clavier

1. **Entree** ou **double-clic** → `StartEdit`
2. Saisir la nouvelle valeur
3. **Entree** → `CommitEdit`
4. **Echap** → `CancelEdit`

## Gestion des evenements

La couche web (`rs-grid-web`) ecoute les evenements `keydown` sur le canvas
et les convertit en valeurs `GridCommand`. Le comportement par defaut du
navigateur est empeche pour les touches gerees, afin d'eviter le defilement
ou d'autres actions indesirables.
