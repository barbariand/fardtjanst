pub mod tables;
use sea_orm::Database;
pub use sea_orm::{self, DatabaseConnection, DbErr};
pub use tables::tempsessions;
pub use tables::notification_info;
pub use tables::users;
pub use tempsessions::Entity as TempSession;
pub use notification_info::Entity as NotificationInfo;
pub use users::Entity as Users;
const DATABASE_URL: &str = "sqlite://sqlite.db";

pub async fn getdb() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    Ok(db)
}
