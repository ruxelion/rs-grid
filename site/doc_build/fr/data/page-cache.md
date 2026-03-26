# Donnees cote serveur (PageCacheDataSource)

## Presentation

`PageCacheDataSource` charge les donnees depuis un serveur distant par pages,
en les mettant en cache avec une eviction LRU. Les cellules pas encore
recuperees s'affichent sous forme de placeholders de chargement (skeleton).

## Creation d'un cache de pages

```rust
use rs_grid_core::page_cache::PageCacheDataSource;

let cache = PageCacheDataSource::new(
    10_000, // total rows (initial estimate)
    100,    // page size (rows per page)
);
cache.set_max_cached_pages(50); // default: 50

let model = GridModel::with_data_source(
    columns,
    Box::new(cache.clone()),
    32.0,
    36.0,
);
model.mode = DataSourceMode::ServerSide;
```

## Cycle de vie d'une page

```
needed_pages() → mark_pending() → fetch → insert_page() → NotifyPageLoaded
```

1. **Detecter les pages necessaires** — appelez `cache.needed_pages(first_row, last_row)`
   pour trouver les pages dans la plage visible qui ne sont ni chargees ni en attente
2. **Marquer comme en attente** — `cache.mark_pending(page_num)` empeche les requetes en double
3. **Recuperer les donnees** — votre application recupere les donnees depuis le serveur
4. **Inserer la page** — `cache.insert_page(page_num, rows)` stocke les donnees
5. **Notifier la grille** — `state.apply(GridCommand::NotifyPageLoaded)` declenche un nouveau rendu

## FetchConfig (recuperation automatique)

`rs-grid-web` fournit un coordinateur de recuperation integre via `FetchConfig` :

```rust
pub struct FetchConfig {
    pub build_url: Box<dyn Fn(&PageFetchRequest) -> String>,
    pub parse_response: Box<dyn Fn(JsValue) -> Result<PageFetchResponse, String>>,
    pub buffer_pages: u64,  // prefetch ahead/behind
}
```

### PageFetchRequest

```rust
pub struct PageFetchRequest {
    pub page_num: u64,
    pub page_size: u64,
    pub sort: Option<SortState>,
    pub filters: HashMap<String, String>,
}
```

Le coordinateur de recuperation effectue automatiquement :

- La verification des pages necessaires pour le viewport actuel
- Le pre-chargement de `buffer_pages` pages en avance et en arriere
- Le lancement d'appels `window.fetch()` asynchrones
- L'analyse des reponses JSON via votre closure `parse_response`
- La mise a jour du cache et le declenchement de `NotifyPageLoaded`

## Reference API

| Methode                             | Description                                            |
| ----------------------------------- | ------------------------------------------------------ |
| `new(total_rows, page_size)`        | Creer un cache vide                                    |
| `page_size()`                       | Obtenir le nombre de lignes par page                   |
| `total_rows()`                      | Obtenir le nombre total de lignes actuel               |
| `set_total_rows(n)`                 | Mettre a jour le total (ex. depuis la reponse serveur) |
| `set_max_cached_pages(n)`           | Definir la limite d'eviction LRU (defaut : 50)         |
| `insert_page(page_num, rows)`       | Stocker une page recuperee                             |
| `clear()`                           | Invalider toutes les pages en cache                    |
| `is_page_loaded(page_num)`          | Verifier si une page est en cache                      |
| `is_page_pending(page_num)`         | Verifier si une requete est en cours                   |
| `mark_pending(page_num)`            | Marquer une page comme en cours de recuperation        |
| `unmark_pending(page_num)`          | Retirer le marqueur d'attente (en cas d'erreur)        |
| `needed_pages(first_row, last_row)` | Pages ni chargees ni en attente dans la plage          |

## CellStatus

`PageCacheDataSource` retourne des valeurs `CellStatus` significatives :

| Statut                     | Quand                                         |
| -------------------------- | --------------------------------------------- |
| `CellStatus::Ready(value)` | Page chargee, la cellule contient des donnees |
| `CellStatus::Loading`      | Page pas encore recuperee                     |
| `CellStatus::Absent`       | Page chargee mais cellule manquante           |

Le renderer dessine une animation skeleton pour les cellules en `Loading`.

## Eviction LRU

Lorsque le cache depasse `max_cached_pages`, les pages les moins recemment
consultees sont evincees. Acceder a nouveau a une page la deplace en fin
de file d'eviction.

## Invalidation lors d'un changement de tri/filtre

Lorsque le tri ou le filtre change en mode server-side, appelez `cache.clear()`
pour invalider toutes les pages, puis laissez le coordinateur re-recuperer
les donnees avec les nouveaux parametres.

## Etat partage

`PageCacheDataSource` utilise `Rc<RefCell<...>>` en interne — le cloner
cree une reference partagee vers le meme cache. C'est ainsi que le coordinateur
de recuperation et le modele de la grille partagent les memes donnees.
