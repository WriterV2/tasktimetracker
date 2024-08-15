#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct TagAssignment {
    pub tgid: i64,
    pub bid: i64,
}
