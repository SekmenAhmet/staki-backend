use bcrypt::{hash, verify, DEFAULT_COST};
use bson::doc;
use chrono::Utc;
use mongodb::{Collection, Database};
use shared::generate_token;

use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};

pub struct AuthService {
    users: Collection<User>,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db: &Database, jwt_secret: String) -> Self {
        Self {
            users: db.collection("users"),
            jwt_secret,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, String> {
        // Vérifier si email existe
        if self
            .users
            .find_one(doc! { "email": &req.email })
            .await
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("Email déjà utilisé".to_string());
        }

        // Hash du password
        let password_hash = hash(&req.password, DEFAULT_COST).map_err(|e| e.to_string())?;

        // Création user
        let user = User {
            id: None,
            email: req.email.clone(),
            username: req.username,
            password_hash,
            created_at: Utc::now(),
        };

        let result = self
            .users
            .insert_one(&user)
            .await
            .map_err(|e| e.to_string())?;
        let user_id = result.inserted_id.as_object_id().unwrap().to_hex();

        // Générer token
        let token =
            generate_token(&user_id, &user.email, &self.jwt_secret).map_err(|e| e.to_string())?;

        Ok(AuthResponse {
            token,
            user: UserResponse {
                id: user_id,
                email: user.email,
                username: user.username,
            },
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, String> {
        // Trouver user
        let user = self
            .users
            .find_one(doc! { "email": &req.email })
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Mauvais identifiants")?;

        // Vérifier password
        if !verify(&req.password, &user.password_hash).map_err(|e| e.to_string())? {
            return Err("Mauvais identifiants".to_string());
        }

        let user_id = user.id.unwrap().to_hex();

        // Générer token
        let token =
            generate_token(&user_id, &user.email, &self.jwt_secret).map_err(|e| e.to_string())?;

        Ok(AuthResponse {
            token,
            user: UserResponse {
                id: user_id,
                email: user.email,
                username: user.username,
            },
        })
    }
}
