use sea_orm;
use sea_orm::entity::prelude::*;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "resor")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[cfg_attr(feature = "with-json", serde(skip_deserializing))]
    pub id: i32,
    pub to_addres: String,
    pub from_addres: String,
    pub by_addres: String,
    pub user_id: Option<i32>,
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
