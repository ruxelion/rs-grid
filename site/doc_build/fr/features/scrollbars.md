# Barres de defilement

## Apercu

rs-grid dessine ses propres barres de defilement sur le canvas au lieu
d'utiliser les barres natives du navigateur. Cela garantit une apparence
coherente entre les plateformes et une integration avec le systeme de theme.

## Composants

Les barres de defilement verticale et horizontale partagent la meme structure :

```
┌──────────┐
│  ▲ fleche│  ← bouton fleche haut/gauche
├──────────┤
│          │
│  piste   │  ← zone de piste cliquable
│          │
│  ▓ curseur│ ← curseur deplacable
│          │
├──────────┤
│  ▼ fleche│  ← bouton fleche bas/droite
└──────────┘
```

## Interactions

| Action                   | Effet                                      |
| ------------------------ | ------------------------------------------ |
| **Glisser le curseur**   | Defilement proportionnel                   |
| **Clic sur la piste**    | Defilement d'une page dans cette direction |
| **Clic sur la fleche**   | Defilement d'un pas fixe                   |
| **Molette de la souris** | `ScrollBy { dx, dy }`                      |

## Geometrie

La geometrie des barres de defilement est precalculee a chaque frame :

### Verticale (`ScrollbarGeom`)

```rust
pub struct ScrollbarGeom {
    pub track_x: f64,     // bord gauche
    pub track_w: f64,     // largeur de la piste
    pub up_btn_y: f64,    // bord superieur du bouton haut
    pub down_btn_y: f64,  // bord superieur du bouton bas
    pub arrow_h: f64,     // hauteur du bouton fleche (= track_w)
    pub track_y: f64,     // haut de la zone defilable
    pub track_h: f64,     // hauteur de la zone defilable
    pub thumb_y: f64,     // bord superieur du curseur
    pub thumb_h: f64,     // hauteur du curseur
}
```

### Horizontale (`HScrollbarGeom`)

Meme structure, refletee horizontalement.

## Visibilite

Les barres de defilement ne s'affichent que lorsque le contenu depasse le viewport :

- Verticale : lorsque `total_height > viewport_height`
- Horizontale : lorsque `total_width > viewport_width`

La taille minimale du curseur est de **24px** pour qu'il reste saisissable.

## Theme

| Variable CSS                 | Description                | Valeur par defaut        |
| ---------------------------- | -------------------------- | ------------------------ |
| `--rs-grid-scrollbar-track`  | Fond de la piste           | `rgb(241,241,241)`       |
| `--rs-grid-scrollbar-thumb`  | Couleur du curseur         | `rgba(100,100,110,0.63)` |
| `--rs-grid-scrollbar-width`  | Largeur de la piste        | `14px`                   |
| `--rs-grid-scrollbar-radius` | Rayon des coins du curseur | `4px`                    |

## Reservation d'espace

La largeur de la barre de defilement est reservee dans le layout :

- `model.scrollbar_size` (par defaut : 14px) est soustrait de l'espace
  disponible pour eviter que le contenu ne soit masque
