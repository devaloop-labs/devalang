# Configuration des Addons Devalang

Ce document explique comment configurer les URLs des API pour les commandes d'addon.

## Variables d'environnement

Les commandes d'addon utilisent les variables d'environnement suivantes :

- `DEVALANG_API_URL` : URL de l'API principale pour les métadonnées (défaut : https://api.devalang.com)
- `DEVALANG_CDN_URL` : URL du CDN pour télécharger les fichiers (défaut : https://cdn.devalang.com)
- `DEVALANG_FORGE_URL` : URL de l'API Forge pour l'authentification (défaut : https://forge.devalang.com)

## Fichier .env

Un fichier `.env` est fourni à la racine du projet avec les valeurs par défaut. Pour utiliser des URLs différentes (par exemple pour le développement local), vous pouvez :

1. Copier le fichier `.env` dans votre répertoire de travail
2. Modifier les valeurs selon vos besoins
3. Décommenter les lignes des URLs locales si nécessaire

Exemple pour le développement local :
```env
DEVALANG_API_URL=http://127.0.0.1:8989
DEVALANG_CDN_URL=http://127.0.0.1:8888
DEVALANG_FORGE_URL=http://127.0.0.1:9090
```

## Commandes disponibles

### Install
Installe un addon depuis le marketplace Devalang.
```bash
devalang addon install publisher.addonname
```

### Remove
Supprime un addon installé.
```bash
devalang addon remove publisher.addonname
```

### List
Liste tous les addons installés.
```bash
devalang addon list
```

### Update
Met à jour un addon vers la dernière version.
```bash
devalang addon update publisher.addonname
```

### Metadata
Affiche les métadonnées d'un addon.
```bash
devalang addon metadata publisher.addonname
```

### Discover
Découvre les addons disponibles (lien vers le marketplace).
```bash
devalang addon discover
```

## Dépendances requises

Les modules addon nécessitent les crates suivants dans `Cargo.toml` :

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
dirs = "5.0"
zip = "0.6"
toml = "0.8"
```

Pour utiliser le fichier `.env` automatiquement (optionnel) :
```toml
dotenv = "0.15"
```

## Architecture

- `metadata.rs` : Récupération des métadonnées depuis l'API
- `download.rs` : Téléchargement et extraction des archives
- `install.rs` : Installation des addons
- `list.rs` : Listage des addons installés
- `remove.rs` : Suppression des addons
- `update.rs` : Mise à jour des addons
- `utils.rs` : Utilitaires pour les URLs signées
- `mod.rs` : Point d'entrée et routing des commandes
