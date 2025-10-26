# Guide d'utilisation de la collection Postman Staki Backend

## Installation

### 1. Importer la collection Postman

1. Ouvrez Postman
2. Cliquez sur **Import** (en haut à gauche)
3. Sélectionnez le fichier `Staki_Backend_API.postman_collection.json`
4. La collection "Staki Backend API" apparaîtra dans votre sidebar

### 2. Importer l'environnement

1. Cliquez sur **Import** à nouveau
2. Sélectionnez le fichier `Staki_Local.postman_environment.json`
3. Sélectionnez l'environnement "Staki - Local" dans le menu déroulant en haut à droite

## Structure de la collection

La collection est organisée en 4 sections principales :

### 1. Auth Service (Port 8081)
- **Register** : Créer un nouveau compte utilisateur
- **Login** : Se connecter et obtenir un token JWT
- **Get Me** : Obtenir les informations de l'utilisateur connecté (requiert authentification)

### 2. Messaging Service (Port 8082)

#### Conversations
- **Create Conversation** : Créer une nouvelle conversation
- **Get Conversation by ID** : Récupérer une conversation spécifique
- **Get User Conversations** : Récupérer toutes les conversations d'un utilisateur
- **Add Participant** : Ajouter un participant à une conversation
- **Remove Participant** : Retirer un participant d'une conversation
- **Delete Conversation** : Supprimer une conversation

#### Messages
- **Send Message** : Envoyer un message dans une conversation
- **Get Message by ID** : Récupérer un message spécifique
- **Get Conversation Messages** : Récupérer tous les messages d'une conversation
- **Mark Message as Read** : Marquer un message comme lu
- **Delete Message** : Supprimer un message

### 3. Social Service (Port 8083)

#### Posts
- **Create Post** : Créer un nouveau post
- **Get Post by ID** : Récupérer un post spécifique
- **Get User Posts** : Récupérer tous les posts d'un utilisateur
- **Delete Post** : Supprimer un post (seulement si vous en êtes l'auteur)

### 4. API Gateway (Port 8080)
- **Register (via Gateway)** : S'inscrire via l'API Gateway
- **Login (via Gateway)** : Se connecter via l'API Gateway
- **Health Check** : Vérifier que l'API Gateway fonctionne

## Workflow de test recommandé

### Scénario 1 : Test complet d'authentification

1. **Register** (Auth Service)
   - Créez un compte avec un email unique
   - Le token sera automatiquement sauvegardé dans `{{auth_token}}`
   - L'ID utilisateur sera sauvegardé dans `{{user_id}}`

2. **Get Me** (Auth Service)
   - Vérifiez que votre token fonctionne
   - Devrait retourner vos informations utilisateur

### Scénario 2 : Test de messagerie

1. **Login** (Auth Service) - Connectez-vous d'abord

2. **Create Conversation** (Messaging Service)
   - Créez une conversation avec d'autres user_ids
   - Le `conversation_id` sera automatiquement sauvegardé

3. **Send Message** (Messaging Service)
   - Envoyez un message dans la conversation créée
   - Le `message_id` sera automatiquement sauvegardé

4. **Get Conversation Messages** (Messaging Service)
   - Récupérez tous les messages de la conversation

5. **Mark Message as Read** (Messaging Service)
   - Marquez le message comme lu

### Scénario 3 : Test de posts sociaux

1. **Login** (Auth Service) - Connectez-vous d'abord

2. **Create Post** (Social Service)
   - Créez un nouveau post
   - Le `post_id` sera automatiquement sauvegardé

3. **Get Post by ID** (Social Service)
   - Récupérez le post que vous venez de créer

4. **Get User Posts** (Social Service)
   - Récupérez tous vos posts

5. **Delete Post** (Social Service)
   - Supprimez le post créé

## Variables d'environnement

Les variables suivantes sont automatiquement gérées :

- `{{base_url}}` : URL de base (http://localhost)
- `{{auth_port}}` : Port du service d'authentification (8081)
- `{{messaging_port}}` : Port du service de messagerie (8082)
- `{{social_port}}` : Port du service social (8083)
- `{{gateway_port}}` : Port de l'API Gateway (8080)
- `{{auth_token}}` : Token JWT (rempli automatiquement après login/register)
- `{{user_id}}` : ID de l'utilisateur connecté (rempli automatiquement)
- `{{conversation_id}}` : ID de la dernière conversation créée
- `{{message_id}}` : ID du dernier message envoyé
- `{{post_id}}` : ID du dernier post créé

## Scripts automatiques

La collection utilise des scripts de test pour automatiser la sauvegarde des variables :

- **Register/Login** : Sauvegarde automatiquement le `auth_token` et `user_id`
- **Create Conversation** : Sauvegarde automatiquement le `conversation_id`
- **Send Message** : Sauvegarde automatiquement le `message_id`
- **Create Post** : Sauvegarde automatiquement le `post_id`

## Authentification

La plupart des endpoints nécessitent un token JWT dans le header `Authorization` :

```
Authorization: Bearer {{auth_token}}
```

Le token est automatiquement ajouté aux requêtes qui en ont besoin après un Register ou Login réussi.

## Démarrage des services

Avant d'utiliser la collection, assurez-vous que tous les services sont démarrés :

```bash
# Démarrer tous les services avec Docker Compose
docker compose up -d

# Ou démarrer chaque service individuellement
cd auth-service && cargo run
cd messaging-service && cargo run
cd social-service && cargo run
cd api-gateway && cargo run
```

## Troubleshooting

### Erreur 401 Unauthorized
- Vérifiez que vous avez un token valide dans `{{auth_token}}`
- Essayez de vous reconnecter avec **Login**

### Erreur 404 Not Found
- Vérifiez que le service correspondant est bien démarré
- Vérifiez que le port est correct dans les variables d'environnement

### Erreur 500 Internal Server Error
- Vérifiez les logs du service
- Vérifiez que MongoDB et Redis sont bien démarrés
- Vérifiez la configuration dans `.env.local`

## Notes importantes

- Les tokens JWT expirent après 15 minutes par défaut
- Les IDs MongoDB utilisent le format ObjectId (24 caractères hexadécimaux)
- Certaines opérations nécessitent d'être le propriétaire de la ressource (ex: supprimer son propre post)
