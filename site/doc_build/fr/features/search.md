# Recherche

## Apercu

Appuyez sur **Ctrl+F** pour ouvrir la barre de recherche. rs-grid parcourt
toutes les cellules visibles a la recherche du terme saisi (correspondance
insensible a la casse, type "contient") et surligne les cellules correspondantes.

## Commandes

| Commande           | Description                                 |
| ------------------ | ------------------------------------------- |
| `Search { query }` | Lance une recherche sur toutes les cellules |
| `SearchNext`       | Passe au resultat suivant                   |
| `SearchPrev`       | Passe au resultat precedent                 |
| `ClearSearch`      | Efface l'etat de recherche et le surlignage |

### Exemple

```rust
state.apply(GridCommand::Search { query: "paris".into() });
state.apply(GridCommand::SearchNext);  // focus sur le résultat suivant
state.apply(GridCommand::SearchPrev); // focus sur le résultat précédent
state.apply(GridCommand::ClearSearch);
```

## SearchState

```rust
pub struct SearchState {
    pub query: String,           // texte de recherche courant
    pub matches: Vec<CellCoord>, // toutes les cellules correspondantes
    pub current: usize,          // index du résultat actif
}
```

## Fonctionnement

`SearchState::run()` parcourt la grille :

1. Itere sur les lignes (jusqu'a **100 000** lignes)
2. Pour chaque ligne, verifie chaque colonne
3. Correspondance insensible a la casse via `contains` sur la valeur de la cellule
4. Collecte jusqu'a **10 000** resultats

## Rendu

Le scene builder utilise deux couleurs du theme pour les resultats de recherche :

| Champ du theme     | Variable CSS                 | Description                                 |
| ------------------ | ---------------------------- | ------------------------------------------- |
| `search_highlight` | `--rs-grid-search-highlight` | Fond pour tous les resultats                |
| `search_current`   | `--rs-grid-search-current`   | Fond pour le resultat actif (avec le focus) |

## Limitations

- **Cote client uniquement** — en mode `ServerSide`, `Search` ne renvoie aucun
  resultat (le volume de donnees est trop important pour un scan local)
- **100 000 lignes parcourues au maximum** — pour eviter de bloquer le thread principal
- **10 000 resultats au maximum** — les resultats supplementaires ne sont pas collectes
- La recherche ne fonctionne pas sur le rendu formate — elle porte sur la valeur brute de la cellule
