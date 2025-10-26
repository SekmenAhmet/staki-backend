# Changelog

## [Unreleased]

### Changed
- **messaging-service**: Mise à jour MongoDB de 2.8 à 3.1
- **messaging-service**: Mise à jour BSON de 2.8 à 2.15
- **messaging-service**: Suppression de la dépendance `redis` 0.32.7 (conflit avec deadpool-redis qui utilise redis 0.23.3)
- **Tous les services**: Adaptation du code pour MongoDB 3.1+ (nouvelle API builder-based)

### Fixed
- **messaging-service**: Correction des appels MongoDB pour compatibilité avec MongoDB 3.1+
  - `find_one(filter, None)` → `find_one(filter)`
  - `insert_one(doc, None)` → `insert_one(doc)`
  - `update_one(filter, update, None)` → `update_one(filter, update)`
  - `delete_one(filter, None)` → `delete_one(filter)`
  - `find(filter, options)` → `find(filter).sort().skip().limit()`
- Résolution des conflits de versions entre les dépendances

### Security
- **messaging-service**: Correction du bug critique d'usurpation d'identité
  - Le `sender_id` est maintenant forcé à partir du JWT utilisateur
  - Validation du contenu des messages (non vide, max 10000 caractères)

### Added
- **messaging-service**: 8 nouvelles routes API
  - `GET /messages/:message_id` - Récupérer un message
  - `PATCH /messages/:message_id/read` - Marquer comme lu
  - `DELETE /messages/:message_id` - Supprimer un message
  - `GET /conversations/:conversation_id` - Récupérer une conversation
  - `DELETE /conversations/:conversation_id` - Supprimer une conversation
  - `POST /conversations/:conversation_id/members` - Ajouter un participant
  - `DELETE /conversations/:conversation_id/members/:user_id` - Retirer un participant
  - `GET /conversations/:conversation_id/messages` - Liste des messages avec pagination
- **messaging-service**: Pagination pour les messages (skip/limit)
- **messaging-service**: Documentation API complète (README.md)
- **Repository**: Suppression de tous les commentaires du code (20 commentaires retirés)
- **Repository**: Nettoyage des dépendances inutilisées (15 dépendances retirées)

### Removed
- **shared**: Dépendances inutilisées: `axum-extra`, `headers`, `async-trait`
- **auth-service**: Dépendances inutilisées: `tower`, `jsonwebtoken`, `serde_json`, `uuid`
- **api-gateway**: Dépendances inutilisées: `tower`, `jsonwebtoken`, `serde`, `serde_json`
- **messaging-service**: Dépendances inutilisées: `tower`, `tower-http`, `uuid`, `tracing`, `jsonwebtoken`, `redis`

## Versions des dépendances principales

### Framework & Runtime
- axum: 0.8.6
- tokio: 1.x (full features)
- tower-http: 0.6.6

### Base de données
- mongodb: 3.1 (unified across all services)
- bson: 2.15 (unified across all services)
- redis: 0.23.3 (via deadpool-redis)
- deadpool-redis: 0.13

### Authentification
- jsonwebtoken: 10.1.0
- bcrypt: 0.17.1

### Utilitaires
- serde: 1.x
- chrono: 0.4
- uuid: 1.x
- anyhow: 1.x
- futures: 0.3

### HTTP Client
- reqwest: 0.12

## Notes de migration

### MongoDB 3.1+
Si vous migrez d'une version antérieure de MongoDB Rust driver:

**Avant (MongoDB 2.x):**
```rust
collection.find_one(filter, None).await?
collection.insert_one(doc, None).await?
collection.find(filter, options).await?
```

**Après (MongoDB 3.1+):**
```rust
collection.find_one(filter).await?
collection.insert_one(doc).await?
collection.find(filter).sort().skip().limit().await?
```

Les options sont maintenant appliquées via des méthodes chainées (builder pattern).

## Build

Le projet compile maintenant sans erreurs:
```bash
cargo build --release
```

Seuls quelques warnings de code mort subsistent dans l'api-gateway (champs de config non utilisés).
