use sea_orm;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel,Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub password: String,
    pub card_nummer: i32,
    pub phone_number: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::tempsessions::Entity")]
    TempSessions,
    #[sea_orm(has_many = "super::notification_info::Entity")]
    NotificationInfo,
}
impl Related<super::tempsessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TempSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
