pub mod Tables;
pub use resor::Entity as Resor;
use sea_orm::Database;
pub use sea_orm::{self, DatabaseConnection, DbErr};
pub use users::Entity as Users;
pub use Tables::resor;
pub use Tables::users;
const DATABASE_URL: &str = "sqlite://sqlite.db";

pub async fn getdb() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    Ok(db)
}
