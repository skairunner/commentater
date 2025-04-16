use sqlx::Postgres;

// Little bit of magic to make a shorthand for sqlx::Acquire
pub trait PgAcquire<'a>: sqlx::Acquire<'a, Database = Postgres> {}
impl<'a, T: sqlx::Acquire<'a, Database = Postgres>> PgAcquire<'a> for T {}
