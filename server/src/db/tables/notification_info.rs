use sea_orm::{self};
use sea_orm::entity::prelude::*;


#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "notificationinfo")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[cfg_attr(feature = "with-json", serde(skip_deserializing))]
    pub id: i32,
    pub user_id:i32,
    pub endpoint:String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
}
impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        
        Relation::Users.def()

    }
}

impl ActiveModelBehavior for ActiveModel {}
