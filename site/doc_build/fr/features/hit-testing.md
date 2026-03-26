# Hit-Testing

## Apercu

Le hit-testing convertit une position du pointeur (en coordonnees viewport)
en cellule, en-tete de colonne ou en-tete de ligne sous le curseur. C'est le
fondement de la selection au clic, du survol, de l'edition et des menus
contextuels.

## Fonctions

### `hit_test(vx, vy, model, scroll_x, scroll_y) → Option<CellCoord>`

Resout une cellule de donnees a partir des coordonnees viewport :

1. **Verification de la gouttiere** — si `vx < row_number_width`, renvoie `None`
2. **Verification de l'en-tete** — si `vy < header_height`, renvoie `None`
3. **Recherche de colonne** — utilise `ColumnOffsets` pour une recherche en O(1)
4. **Recherche de ligne** — O(1) grace a la hauteur de ligne uniforme

Renvoie `None` si le pointeur se trouve sous la derniere ligne ou a droite de
la derniere colonne.

### `hit_test_col_header(vx, vy, model, scroll_x) → Option<usize>`

Renvoie l'index de la colonne lors d'un clic sur un en-tete de colonne. Utilise pour :

- Basculer le tri
- Selection de colonne
- Detection du redimensionnement de colonne

### `hit_test_row_header(vx, vy, model, scroll_y) → Option<u64>`

Renvoie l'index de la ligne lors d'un clic sur la gouttiere des numeros de ligne. Utilise pour :

- Selection de ligne
- Extension de la selection de ligne (glisser)

## Colonnes epinglees

Le hit-testing prend en compte les colonnes epinglees :

```
if vx_data < pinned_width:
    abs_x = vx_data          // epinglee : pas de decalage de scroll
else:
    abs_x = vx_data + scroll_x  // defilable : ajout du scroll
```

Cela garantit que les clics sur les colonnes epinglees sont resolus correctement,
quelle que soit la position du defilement horizontal.

## Precision aux positions de defilement extremes

Pour les jeux de donnees tres volumineux (milliards de lignes), un calcul naif
`(vy + scroll_y - header_height) / row_height` perd en precision f64 car il
soustrait deux grands nombres.

Le code de hit-testing utilise une **decomposition preservant la precision** :

```rust
let sy_content = scroll_y - header_height;
let first_row = (sy_content / row_height) as u64;
let frac = sy_content % row_height;  // decalage infra-ligne
let offset = ((vy + frac) / row_height) as u64;
let row = first_row + offset;
```

Cela maintient les valeurs intermediaires petites, preservant la precision meme
pour des indices de ligne proches de la limite u64.

## Performance

- Recherche de colonne : O(1) via les `ColumnOffsets` precalcules
- Recherche de ligne : O(1) avec une hauteur de ligne uniforme
- Total : O(1) par appel de hit-test

:::warning
Ne jamais introduire d'algorithme en O(n) dans le chemin de hit-testing. Les
offsets de colonnes sont precalcules une seule fois lors d'un changement de
colonnes, et non a chaque evenement pointeur.
:::
