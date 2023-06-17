use sea_orm::entity::prelude::*;
use sea_orm;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "notificationinfo")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[cfg_attr(feature = "with-json", serde(skip_deserializing))]
    pub id: i32,
    pub UserId: i32,
    pub endpoint: String,
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
impl From<Model> for web_push::SubscriptionInfo{
    fn from(value: Model) -> Self {
        Self::new(value.endpoint, value.p256dh, value.auth)
    }
}
pub struct LinkedSubscriptionInfo{
    subscription_info:web_push::SubscriptionInfo,
    user:super::users::Model,
}
impl LinkedSubscriptionInfo{
    pub fn new(user: super::users::Model,subscription_info:web_push::SubscriptionInfo)->Self{
        Self { subscription_info, user }
    }
}
impl From<LinkedSubscriptionInfo> for ActiveModel {
    fn from(value:LinkedSubscriptionInfo) -> Self {
        Self {
            UserId: sea_orm::ActiveValue::Set(value.user.id),
            endpoint: sea_orm::ActiveValue::Set(value.subscription_info.endpoint),
            auth: sea_orm::ActiveValue::Set(value.subscription_info.keys.auth),
            p256dh: sea_orm::ActiveValue::Set(value.subscription_info.keys.p256dh),
            ..Default::default()
        }
    }
}


