pub mod tables;
use sea_orm::Database;
pub use sea_orm::{self, DatabaseConnection, DbErr};
pub use tables::tempsessions;
pub use tables::users;
pub use tables::resor;
pub use tables::Resor;
pub use tables::Users;
pub use tables::TempSession;
const DATABASE_URL: &str = "sqlite://sqlite.db";

pub async fn getdb() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    Ok(db)
}
