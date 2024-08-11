#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Booking {
    pub id: i64,
    pub startdate: i64,
    pub enddate: Option<i64>,
    pub des: String,
}
