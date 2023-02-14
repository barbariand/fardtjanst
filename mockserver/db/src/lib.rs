pub mod cake;
pub use cake::Entity as Cake;
pub mod fruit;
pub use fruit::Entity as Fruit;
use sea_orm::Database;
pub use sea_orm::{self, DatabaseConnection, DbErr};
const DATABASE_URL: &str = "sqlite://sqlite.db";

pub async fn getdb() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    Ok(db)
}
