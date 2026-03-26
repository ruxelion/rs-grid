# API ColumnDef

## ColumnDef

```rust
pub struct ColumnDef {
    pub key: String,
    pub label: String,
    pub width: f64,
    pub format: Option<CellFormat>,
    pub editor: Option<CellEditor>,
    pub validator: Option<CellValidator>,
}
```

| Champ       | Type                    | Description                                                         |
| ----------- | ----------------------- | ------------------------------------------------------------------- |
| `key`       | `String`                | Identifiant unique, utilisé pour rechercher les valeurs de cellules |
| `label`     | `String`                | Texte affiché dans l'en-tête de colonne                             |
| `width`     | `f64`                   | Largeur en pixels logiques                                          |
| `format`    | `Option<CellFormat>`    | Format d'affichage (`None` = texte brut)                            |
| `editor`    | `Option<CellEditor>`    | Type d'éditeur (`None` = champ texte par défaut)                    |
| `validator` | `Option<CellValidator>` | Validateur optionnel appelé avant la validation d'une édition       |

### Constructeur

```rust
pub fn new(key: impl Into<String>, label: impl Into<String>, width: f64) -> Self
```

Crée une colonne sans format et sans surcharge d'éditeur.

## CellFormat

```rust
#[non_exhaustive]
pub enum CellFormat {
    Number { decimal_places: u8, thousands_sep: Option<char>, decimal_sep: char },
    Percent { decimal_places: u8 },
    Currency { symbol: String, decimal_places: u8, thousands_sep: Option<char>, symbol_after: bool },
    Boolean { true_label: String, false_label: String },
    Custom(Rc<dyn Fn(&str) -> FormattedCell>),
    Image { base_url: Option<String>, border_radius: f64, padding: f64 },
    ImageText { base_url: String, suffix: String, image_size: f64, border_radius: f64, gap: f64 },
}
```

### Méthodes

| Méthode           | Retour | Description                       |
| ----------------- | ------ | --------------------------------- |
| `is_image()`      | `bool` | Vrai pour la variante `Image`     |
| `is_image_text()` | `bool` | Vrai pour la variante `ImageText` |

## FormattedCell

```rust
pub struct FormattedCell {
    pub text: String,
    pub align: Option<CellAlign>,
    pub bold: bool,
    pub color: Option<[u8; 4]>,  // RGBA
}
```

## CellAlign

```rust
pub enum CellAlign {
    Left,    // par défaut
    Center,
    Right,
}
```

## CellValidator

```rust
pub struct CellValidator(pub Rc<dyn Fn(&str) -> Result<(), String>>);
```

Callback de validation par colonne. Utilisez `CellValidator::new` pour en créer un :

```rust
CellValidator::new(|v| {
    v.parse::<f64>()
        .map(|_| ())
        .map_err(|_| "pas un nombre".to_string())
})
```

| Méthode                 | Description                                                 |
| ----------------------- | ----------------------------------------------------------- |
| `new(f)`                | Encapsule une closure en validateur                         |
| `validate(value: &str)` | Exécute le validateur ; retourne `Ok(())` ou `Err(message)` |

Voir [Validation](/fr/features/validation.md) pour le guide complet.

## CellEditor

```rust
#[non_exhaustive]
pub enum CellEditor {
    Text,
    Select { options: Vec<SelectOption> },
}
```

## SelectOption

```rust
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub icon: Option<String>,
}
```

## ColumnOffsets

```rust
pub struct ColumnOffsets {
    pub offsets: Vec<f64>,
    pub total_width: f64,
}
```

| Méthode                                     | Description                                             |
| ------------------------------------------- | ------------------------------------------------------- |
| `compute(columns: &[ColumnDef])`            | Construit les offsets à partir des largeurs de colonnes |
| `hit_column(x: f64, columns: &[ColumnDef])` | Trouve la colonne à la position x                       |
