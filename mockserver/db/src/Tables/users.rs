use sea_orm;
use sea_orm::entity::prelude::*;
#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub password: String,
    pub card_nummer: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::resor::Entity")]
    Resor,
}
impl Related<super::resor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Resor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
