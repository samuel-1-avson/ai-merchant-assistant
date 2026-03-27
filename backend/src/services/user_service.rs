use std::sync::Arc;
use uuid::Uuid;

use crate::db::repositories::user_repo::UserRepository;
use crate::models::user::{User, CreateUserRequest};

pub struct UserService {
    user_repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn get_user(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_id(id).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_email(email).await
    }
}
