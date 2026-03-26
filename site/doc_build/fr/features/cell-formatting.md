# Formatage des cellules

## Vue d'ensemble

`CellFormat` contrôle l'affichage des valeurs brutes des cellules. C'est purement
visuel — les données sous-jacentes ne sont pas modifiées. Il se définit via `ColumnDef::format` :

```rust
col.format = Some(CellFormat::Number {
    decimal_places: 2,
    thousands_sep: Some(' '),
    decimal_sep: '.',
});
```

## Types de format

### Number

```rust
CellFormat::Number {
    decimal_places: u8,           // chiffres après la virgule
    thousands_sep: Option<char>,  // ex. Some(' ') ou Some(',')
    decimal_sep: char,            // ex. '.' ou ','
}
```

| Brut         | Format                                                          | Résultat     | Alignement |
| ------------ | --------------------------------------------------------------- | ------------ | ---------- |
| `"1234.567"` | `decimal_places: 2, thousands_sep: Some(' '), decimal_sep: '.'` | `"1 234.57"` | Droite     |
| `"1234.5"`   | `decimal_places: 2, thousands_sep: Some('.'), decimal_sep: ','` | `"1.234,50"` | Droite     |

### Percent

```rust
CellFormat::Percent {
    decimal_places: u8,
}
```

Multiplie par 100 et ajoute `%` :

| Brut     | Résultat   |
| -------- | ---------- |
| `"0.75"` | `"75.00%"` |

### Currency

```rust
CellFormat::Currency {
    symbol: String,               // ex. "$" ou "€"
    decimal_places: u8,
    thousands_sep: Option<char>,
    symbol_after: bool,           // true → "42.50 €", false → "$42.50"
}
```

| Brut       | symbol\_after | Résultat       |
| ---------- | ------------- | -------------- |
| `"42.5"`   | `false`       | `"$42.50"`     |
| `"1234.5"` | `true`        | `"1 234.50 €"` |

### Boolean

```rust
CellFormat::Boolean {
    true_label: String,    // ex. "✓" ou "Oui"
    false_label: String,   // ex. "✗" ou "Non"
}
```

Valeurs considérées vraies : `"true"`, `"1"`, `"yes"` (insensible à la casse).
Tout le reste est considéré faux. Affiché centré.

### Image

```rust
CellFormat::Image {
    base_url: Option<String>,  // préfixe d'URL (None = la valeur brute est l'URL complète)
    border_radius: f64,        // rayon des coins en px
    padding: f64,              // padding à l'intérieur de la cellule
}
```

Affiche la valeur de la cellule sous forme d'image. L'URL finale est `base_url + raw_value`.

### ImageText

```rust
CellFormat::ImageText {
    base_url: String,      // ex. "https://flagcdn.com/w40/"
    suffix: String,        // ex. ".png"
    image_size: f64,       // taille carrée de l'image en px
    border_radius: f64,
    gap: f64,              // espace entre l'image et le texte
}
```

Format de la valeur brute : `"CLÉ Libellé"` — le premier token est la clé de
l'image (mise en minuscules pour l'URL), le reste est le texte affiché.

Exemple : `"FR France"` → URL de l'image `https://flagcdn.com/w40/fr.png` + texte `"France"`

### Custom

```rust
CellFormat::Custom(Rc::new(|raw: &str| FormattedCell {
    text: raw.to_uppercase(),
    bold: true,
    color: Some([255, 0, 0, 255]),  // rouge
    ..Default::default()
}))
```

Contrôle total sur le texte, l'alignement, le gras et la couleur via un callback.

## Sortie FormattedCell

Tous les formats produisent un `FormattedCell` :

```rust
pub struct FormattedCell {
    pub text: String,
    pub align: Option<CellAlign>,  // Left, Center, Right
    pub bold: bool,
    pub color: Option<[u8; 4]>,    // surcharge RGBA
}
```

## Comportement de repli

Si la valeur brute ne peut pas être analysée (ex. `"abc"` pour un format Number),
la valeur brute est affichée telle quelle, sans formatage.
