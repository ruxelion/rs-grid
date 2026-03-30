# Serveur MCP

## Vue d'ensemble

rs-grid fournit un **serveur Model Context Protocol (MCP)** publié sur npm sous le
nom [`rs-grid-mcp`](https://www.npmjs.com/package/rs-grid-mcp). C'est un serveur
TypeScript stdio qui expose la documentation complète à Claude Code et à tout autre
agent IA compatible MCP.

Une fois enregistré, l'agent peut chercher dans la documentation, lire des pages
individuelles et accéder à l'index de contexte complet — sans quitter la conversation.

## Configuration

### Dans votre projet (npx)

Aucune installation requise. Ajoutez un fichier `.mcp.json` à la racine de votre
projet :

```json title=".mcp.json"
{
  "mcpServers": {
    "rs-grid-docs": {
      "command": "npx",
      "args": ["-y", "rs-grid-mcp"]
    }
  }
}
```

Claude Code détecte automatiquement `.mcp.json` à l'ouverture du projet et demande
une approbation unique. `npx -y` télécharge et exécute la dernière version
automatiquement.

Pour **VS Code Copilot**, créez `.vscode/mcp.json` à la place :

```json title=".vscode/mcp.json"
{
  "servers": {
    "rs-grid-docs": {
      "command": "npx",
      "args": ["-y", "rs-grid-mcp"],
      "type": "stdio"
    }
  }
}
```

**Enregistrement via CLI** (niveau utilisateur, tous les projets) :

```bash
claude mcp add rs-grid-docs -- npx -y rs-grid-mcp
```

### Dans le dépôt rs-grid (contributeurs)

Le dépôt contient un `.mcp.json` préconfiguré pointant vers le build local.
Compilez le serveur une fois, Claude Code le détecte ensuite automatiquement :

```bash
just mcp-build
```

:::tip
Pour travailler sur le code source du MCP lui-même, utilisez `just mcp-dev` pour
exécuter via `tsx` sans étape de compilation — les modifications prennent effet au
prochain redémarrage du serveur.
:::

## Outils disponibles

| Outil                 | Paramètres                                                                                 | Description                                                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `search_rs_grid_docs` | `query` (string), `limit` (number, défaut 5), `language` (`"en"` \| `"fr"`, défaut `"en"`) | Recherche par mots-clés dans toutes les pages de documentation. Retourne des extraits classés par score, centrés sur la première occurrence. |

## Ressources disponibles

| URI                       | Description                                                            |
| ------------------------- | ---------------------------------------------------------------------- |
| `rs-grid://llms.txt`      | Index de la documentation — titres et descriptions de toutes les pages |
| `rs-grid://llms-full.txt` | Documentation complète concaténée (\~5 000 lignes)                     |
| `rs-grid://skill.md`      | Capacités, contraintes et workflows pour les agents IA                 |
| `rs-grid://docs/{path}`   | Page individuelle — ex. `rs-grid://docs/api/grid-state.md`             |
