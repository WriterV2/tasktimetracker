#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
