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
    pub by_addres: Option<String>,
    pub user_id: Option<i32>,
    pub time: i64,
    pub passagers: i32,
    pub child_passagers: i32,
    pub is_shared: bool,
    pub can_be_new_trip_template: bool,
    pub transport: String,
    pub from_id: i32,
    pub by_id: Option<i32>,
    pub to_id: i32,
    pub cancelleable: bool,
    pub company_name: Option<String>,
    pub status: Option<String>,
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
