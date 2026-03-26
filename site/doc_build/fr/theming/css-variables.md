# Reference des variables CSS

## Fonctionnement

rs-grid lit les proprietes CSS personnalisees depuis `:root` au montage via
`theme_from_css_vars()`. Chaque variable correspond a un champ de la structure
`Theme`. Si une variable est absente ou non analysable, la valeur par defaut
du theme clair est utilisee.

## Formats de couleur supportes

- `#rrggbb` / `#rrggbbaa`
- `#rgb` / `#rgba`
- `rgb(r, g, b)`
- `rgba(r, g, b, a)` — `a` est un flottant entre 0 et 1

## Variables de couleur

| Variable CSS                 | Champ Theme        | Defaut clair             | Defaut sombre            | Description                                |
| ---------------------------- | ------------------ | ------------------------ | ------------------------ | ------------------------------------------ |
| `--rs-grid-bg`               | `bg`               | `#ffffff`                | `rgb(26,27,38)`          | Arriere-plan des cellules                  |
| `--rs-grid-header-bg`        | `header_bg`        | `rgb(248,249,250)`       | `rgb(36,40,59)`          | Arriere-plan des en-tetes                  |
| `--rs-grid-header-text`      | `header_text`      | `rgb(24,29,31)`          | `rgb(169,177,214)`       | Couleur du texte des en-tetes              |
| `--rs-grid-cell-text`        | `cell_text`        | `rgb(24,29,31)`          | `rgb(192,202,245)`       | Couleur du texte des cellules              |
| `--rs-grid-grid-line`        | `grid_line`        | `rgb(224,224,224)`       | `rgb(42,47,69)`          | Couleur des bordures de cellules           |
| `--rs-grid-header-border`    | `header_border`    | `rgb(186,191,199)`       | `rgb(61,68,102)`         | Bordure inferieure des en-tetes            |
| `--rs-grid-selection-fill`   | `selection_fill`   | `rgba(31,119,220,0.18)`  | `rgba(122,162,255,0.2)`  | Remplissage du rectangle de selection      |
| `--rs-grid-selection-border` | `selection_border` | `rgba(31,119,220,0.82)`  | `rgba(122,162,255,0.8)`  | Bordure du rectangle de selection          |
| `--rs-grid-scrollbar-track`  | `scrollbar_track`  | `rgb(241,241,241)`       | `rgb(31,35,53)`          | Arriere-plan de la piste de la scrollbar   |
| `--rs-grid-scrollbar-thumb`  | `scrollbar_thumb`  | `rgba(100,100,110,0.63)` | `rgba(169,177,214,0.4)`  | Couleur du curseur de la scrollbar         |
| `--rs-grid-row-alt-bg`       | `row_alt_bg`       | `rgb(252,252,253)`       | `rgb(30,32,48)`          | Arriere-plan des lignes alternees          |
| `--rs-grid-row-hover-bg`     | `row_hover_bg`     | `rgba(0,0,0,0.04)`       | `rgba(255,255,255,0.04)` | Superposition au survol des lignes         |
| `--rs-grid-search-highlight` | `search_highlight` | `rgba(255,213,0,0.31)`   | `rgba(255,213,0,0.31)`   | Surlignage des resultats de recherche      |
| `--rs-grid-search-current`   | `search_current`   | `rgba(255,165,0,0.55)`   | `rgba(255,165,0,0.55)`   | Resultat de recherche actuel               |
| `--rs-grid-skeleton-fg`      | `skeleton_fg`      | `rgba(200,200,200,0.39)` | `rgba(60,65,90,0.39)`    | Couleur de la barre de chargement skeleton |

## Variables de taille et typographie

| Variable CSS                 | Champ Theme        | Defaut | Type | Description                                       |
| ---------------------------- | ------------------ | ------ | ---- | ------------------------------------------------- |
| `--rs-grid-font-size`        | `font_size`        | `14px` | px   | Taille de police des cellules                     |
| `--rs-grid-header-font-size` | `header_font_size` | `12px` | px   | Taille de police des en-tetes                     |
| `--rs-grid-header-font-bold` | `header_font_bold` | `true` | bool | En-tetes en gras (`true`/`false`/`1`/`0`)         |
| `--rs-grid-cell-padding`     | `cell_padding`     | `10px` | px   | Padding horizontal des cellules                   |
| `--rs-grid-scrollbar-width`  | `scrollbar_width`  | `14px` | px   | Largeur de la piste et du curseur de la scrollbar |
| `--rs-grid-scrollbar-radius` | `scrollbar_radius` | `4px`  | px   | Rayon des coins du curseur de la scrollbar        |

## Exemple : theme sombre personnalise

```css
:root {
  --rs-grid-bg: #1e1e2e;
  --rs-grid-header-bg: #313244;
  --rs-grid-header-text: #cdd6f4;
  --rs-grid-cell-text: #cdd6f4;
  --rs-grid-grid-line: #45475a;
  --rs-grid-header-border: #585b70;
  --rs-grid-selection-fill: rgba(137, 180, 250, 0.2);
  --rs-grid-selection-border: rgba(137, 180, 250, 0.8);
  --rs-grid-scrollbar-track: #1e1e2e;
  --rs-grid-scrollbar-thumb: rgba(166, 173, 200, 0.4);
  --rs-grid-row-alt-bg: #181825;
  --rs-grid-row-hover-bg: rgba(255, 255, 255, 0.04);
  --rs-grid-font-size: 13px;
  --rs-grid-header-font-size: 12px;
}
```

:::note
Les variables CSS sont lues **une seule fois** au montage. Les modifier
dynamiquement necessite de remonter la grille ou de passer une structure
`Theme` mise a jour de maniere programmatique via un signal Leptos.
:::
