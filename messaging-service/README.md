# Messaging Service

Service de messagerie en temps réel pour l'application Staki.

## Fonctionnalités

- Envoi et réception de messages
- Gestion des conversations (privées et groupes)
- Marquage des messages comme lus
- Gestion des participants dans les conversations
- Cache Redis pour améliorer les performances
- Authentification JWT

## Configuration

Créez un fichier `.env` avec les variables suivantes:

```env
PORT=8002
MONGO_URI=mongodb://localhost:27017
REDIS_URI=redis://localhost:6379
JWT_SECRET=your_secret_key_here
```

## API Endpoints

### Messages

#### POST /messages
Envoyer un nouveau message

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Body:**
```json
{
  "conversation_id": "507f1f77bcf86cd799439011",
  "content": "Hello, world!"
}
```

**Response:** `201 Created`
```json
{
  "message_id": "507f1f77bcf86cd799439012",
  "sent_at": "2025-01-01T12:00:00Z"
}
```

#### GET /conversations/:conversation_id/messages?skip=0&limit=50
Récupérer les messages d'une conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Query Parameters:**
- `skip` (optional): Nombre de messages à ignorer (pagination), défaut: 0
- `limit` (optional): Nombre maximum de messages à retourner, défaut: 50, max: 100

**Response:** `200 OK`
```json
[
  {
    "id": "507f1f77bcf86cd799439012",
    "conversation_id": "507f1f77bcf86cd799439011",
    "sender_id": "user123",
    "content": "Hello, world!",
    "sent_at": "2025-01-01T12:00:00Z",
    "read": false
  }
]
```

#### GET /messages/:message_id
Récupérer un message spécifique

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
{
  "id": "507f1f77bcf86cd799439012",
  "conversation_id": "507f1f77bcf86cd799439011",
  "sender_id": "user123",
  "content": "Hello, world!",
  "sent_at": "2025-01-01T12:00:00Z",
  "read": false
}
```

#### PATCH /messages/:message_id/read
Marquer un message comme lu

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
"Message marked as read"
```

#### DELETE /messages/:message_id
Supprimer un message (seulement l'expéditeur peut supprimer)

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
"Message deleted"
```

### Conversations

#### POST /conversations
Créer une nouvelle conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Body:**
```json
{
  "participants": ["user123", "user456"]
}
```

**Response:** `201 Created`
```json
{
  "id": "507f1f77bcf86cd799439011",
  "participants": ["user123", "user456"],
  "created_at": "2025-01-01T12:00:00Z",
  "updated_at": "2025-01-01T12:00:00Z"
}
```

#### GET /conversations/:conversation_id
Récupérer les détails d'une conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
{
  "id": "507f1f77bcf86cd799439011",
  "participants": ["user123", "user456"],
  "created_at": "2025-01-01T12:00:00Z",
  "updated_at": "2025-01-01T12:00:00Z"
}
```

#### GET /users/:user_id/conversations
Récupérer toutes les conversations d'un utilisateur

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
[
  {
    "id": "507f1f77bcf86cd799439011",
    "participants": ["user123", "user456"],
    "created_at": "2025-01-01T12:00:00Z",
    "updated_at": "2025-01-01T12:00:00Z"
  }
]
```

#### DELETE /conversations/:conversation_id
Supprimer une conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
"Conversation deleted"
```

#### POST /conversations/:conversation_id/members
Ajouter un participant à une conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Body:**
```json
{
  "user_id": "user789"
}
```

**Response:** `200 OK`
```json
"Participant added"
```

#### DELETE /conversations/:conversation_id/members/:user_id
Retirer un participant d'une conversation

**Headers:**
- `Authorization: Bearer <JWT_TOKEN>`

**Response:** `200 OK`
```json
"Participant removed"
```

## Validation & Sécurité

### Messages
- Le contenu ne peut pas être vide
- Longueur maximale du contenu: 10 000 caractères
- Le `sender_id` est automatiquement défini à partir du JWT (empêche l'usurpation d'identité)
- Seuls les participants d'une conversation peuvent envoyer/lire des messages

### Conversations
- Les participants doivent être fournis lors de la création
- L'utilisateur créateur est automatiquement ajouté aux participants
- Seuls les participants peuvent voir une conversation
- Seul l'utilisateur lui-même ou un participant peut retirer un membre

## Cache Redis

Les messages d'une conversation sont mis en cache pendant 60 secondes. Le cache est invalidé lors de:
- L'envoi d'un nouveau message
- Le marquage d'un message comme lu
- La suppression d'un message

## Pagination

Pour récupérer les messages d'une conversation:
- Par défaut: 50 messages les plus récents
- Maximum: 100 messages par requête
- Utilisez `skip` et `limit` pour la pagination

Exemple: `/conversations/507f1f77bcf86cd799439011/messages?skip=50&limit=50`

## Améliorations Futures

- [ ] WebSocket pour les notifications en temps réel
- [ ] Support des pièces jointes (images, fichiers)
- [ ] Réactions aux messages (emojis)
- [ ] Threads de réponses
- [ ] Recherche full-text dans les messages
- [ ] Statut "en train d'écrire"
- [ ] Indicateur de présence (en ligne/hors ligne)
- [ ] Suppression logique au lieu de suppression physique
- [ ] Édition de messages
- [ ] Messages épinglés
- [ ] Notifications push

## Dépendances Principales

- `axum` - Framework web
- `tokio` - Runtime async
- `mongodb` - Base de données
- `redis` / `deadpool-redis` - Cache et pub/sub
- `serde` / `serde_json` - Sérialisation
- `chrono` - Gestion des dates
- `shared` - Authentification JWT partagée
