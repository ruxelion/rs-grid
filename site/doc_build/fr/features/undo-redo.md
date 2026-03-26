# Annuler et Retablir

## Apercu

rs-grid maintient un historique d'annulation/retablissement pour les actions
reversibles. Appuyez sur **Ctrl+Z** pour annuler et **Ctrl+Y** pour retablir.

## Commandes

```rust
state.apply(GridCommand::Undo);
state.apply(GridCommand::Redo);
```

## Actions suivies

| Action                                        | Type d'entree undo                                  |
| --------------------------------------------- | --------------------------------------------------- |
| Edition de cellule (`CommitEdit`)             | `SetCell` — restaure la valeur precedente           |
| Coller (`PasteAt`)                            | `SetCells` — restaure toutes les cellules affectees |
| Couper (`CutSelection`)                       | `SetCells` — restaure les cellules effacees         |
| Redimensionnement de colonne (`ResizeColumn`) | `ResizeColumn` — restaure l'ancienne largeur        |
| Deplacement de colonne (`MoveColumn`)         | `MoveColumn` — inverse les indices from/to          |

## Capacite de l'historique

La pile d'annulation contient au maximum **100 entrees**. Lorsque la limite est
atteinte, l'entree la plus ancienne est supprimee (FIFO). Toute nouvelle action
annulable vide entierement la pile de retablissement.

## Fonctionnement

1. Lorsqu'une commande annulable est appliquee, `GridState` empile une
   `UndoEntry` contenant l'operation inverse
2. `Undo` depile la pile d'annulation, applique l'inverse et empile
   l'inverse inverse sur la pile de retablissement
3. `Redo` depile la pile de retablissement, applique l'entree et la
   reempile sur la pile d'annulation

```
[Pile undo]  ←→  [Pile redo]
   push ←── nouvelle action (vide la pile redo)
   pop  ──→ appliquer l'inverse ──→ push vers redo
             redo pop ──→ appliquer ──→ push vers undo
```

## Actions NON suivies

Ces actions ne sont pas annulables :

- Changements de selection
- Defilement
- Tri / filtrage
- Recherche
- Etat de survol
