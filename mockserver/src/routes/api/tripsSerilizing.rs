#![allow(non_snake_case)] //beacuse needs serialzieble to json with CamelCaseNames :)

use chrono::{TimeZone, Utc,Duration, DateTime};
use crate::db as db;
use db::{
    resor,
    sea_orm::{
        Condition, DbBackend, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select,
    },
    users,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::{str::FromStr};
#[derive(Serialize, Deserialize, Clone, Debug)]
enum Transport {
    #[allow(non_camel_case_types)]
    taxi,
    #[allow(non_camel_case_types)]
    wheelChairTaxi,
}
impl ToString for Transport {
    fn to_string(&self) -> String {
        match self {
            Self::taxi => String::from("Taxi"),
            Self::wheelChairTaxi => String::from("Wheel Chair Taxi"),
        }
    }
}
impl FromStr for Transport {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Taxi" => Ok(Self::taxi),
            "Wheel Chair Taxi" => Ok(Self::wheelChairTaxi),
            _ => Err(()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
enum Company {
    TaxiKurir,
    ArlandaExpress,
    EkeröTaxi,
    HaningeNynäshamnTaxi,
    Samtrans,
    Sirius,
    SverigeTaxi,
    SödertäljeTaxi,
    TaxiStockholm,
}
impl ToString for Company {
    fn to_string(&self) -> String {
        match self {
            Self::TaxiKurir => String::from("Taxi Kurir"),
            Self::ArlandaExpress => String::from("Arlanda Express"),
            Self::EkeröTaxi => String::from("Ekerö Taxi"),
            Self::HaningeNynäshamnTaxi => String::from("Haninge & Nynäshamn Taxi"),
            Self::Samtrans => String::from("Samtrans"),
            Self::Sirius => String::from("Sirius"),
            Self::SverigeTaxi => String::from("Sverigetaxi"),
            Self::SödertäljeTaxi => String::from("Södertälje Taxi"),
            Self::TaxiStockholm => String::from("Taxi Stockholm"),
        }
    }
}
impl FromStr for Company {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Taxi Kurir" => Ok(Self::TaxiKurir),
            "Arlanda Express" => Ok(Self::ArlandaExpress),
            "Ekerö Taxi" => Ok(Self::EkeröTaxi),
            "Haninge & Nynäshamn Taxi" => Ok(Self::HaningeNynäshamnTaxi),
            "Samtrans" => Ok(Self::Samtrans),
            "Sirius" => Ok(Self::Sirius),
            "Sverigetaxi" => Ok(Self::SverigeTaxi),
            "Södertälje Taxi" => Ok(Self::SödertäljeTaxi),
            "Taxi Stockholm" => Ok(Self::TaxiStockholm),
            _ => Err(()),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Address {
    id: String,
    address: String,
}
impl Address {
    fn new(id: i32, address: String) -> Address {
        Address {
            id: id.to_string(),
            address,
        }
    }
}

#[derive(Serialize, Deserialize, Debug,Clone,PartialEq, Eq)]
pub enum ReservationStatusEnum {
    ResaBesäld,
    BilPåväg,
    LetarEfterBil
}
impl ToString for ReservationStatusEnum {
    fn to_string(&self) -> String {
        match self {
            Self::ResaBesäld => String::from("Resa Bestäld"),
            Self::BilPåväg => String::from("Bil Påväg"),
            Self::LetarEfterBil=> String::from("Letar efter bil")
        }
    }
}
impl FromStr for ReservationStatusEnum {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Resa Beställd" => Ok(Self::ResaBesäld),
            "Bil påväg" => Ok(Self::BilPåväg),
            _ => Err(()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ReservationStatus {
    pub status: ReservationStatusEnum,
}
fn is_none_or_empty(op: &Option<String>) -> bool {
    !match op {
        Some(s) => s.eq(""),
        None => false,
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct CustomerInfo {
    phoneNumber: String,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hasReservationStatus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservationStatus: Option<ReservationStatus>,
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Departure {
    id: String,
    transport: Transport,
    transportProvider: Option<Company>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transportProviderName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transportId: Option<String>,
    canBeCancelled: bool,
    departure: String,
    from: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    by: Option<Address>,
    to: Address,
    pub customerInfo: CustomerInfo,
}
impl Departure {
    pub fn get_time(&self)->Option<DateTime<Utc>>{
        DateTime::from_str(&self.departure).ok()
    }
    pub fn set_status(&mut self,s:ReservationStatus){
        self.customerInfo.reservationStatus=Some(s);
        self.customerInfo.hasReservationStatus=Some(true);
    }
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: String,
        transport_string: String,
        transport_provider_name: Option<String>,
        transport_id: String,
        departure: String,
        can_be_cancelled: bool,
        from: Address,
        by: Option<Address>,
        to: Address,
        customer_info: CustomerInfo,
    ) -> Result<Departure, String> {
        let transport = match Transport::from_str(&transport_string) {
            Ok(t) => t,
            Err(_) => return Err("failed to find transport".to_string()),
        };
        let transport_provider = match transport_provider_name.clone() {
            Some(s) => match Company::from_str(&s) {
                Ok(company) => Some(company),
                Err(_) => return Err("failed to parse company name".to_string()),
            },
            None => None,
        };
        Ok(Departure {
            id,
            transport,
            transportProvider: transport_provider,
            transportProviderName: transport_provider_name,
            transportId: Some(transport_id),
            canBeCancelled: can_be_cancelled,
            departure,
            from,
            by,
            to,
            customerInfo: customer_info,
        })
    }
}

#[derive(Serialize, Deserialize, Debug,Clone)]

pub struct Trips {
    id: String,
    customerName: String,
    customerCardNumber: String,
    phoneNumber: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    allocationId: String,
    transports: Vec<Transport>,
    transport: Transport,
    from: Address,
    by: Option<Address>,
    to: Address,
    pub departure: Departure,
    passengers: i32,
    childPassengers: i32,
    attributes: Vec<String>,
    appliances: Vec<String>,
    isShared: bool,
    canBeNewTripTemplate: bool,
    canBeCancelled: bool,
}
impl Trips {
    pub fn try_new(user: &users::Model, resor: &resor::Model) -> Result<Trips, String> {
        let datetime = match Utc.timestamp_millis_opt(resor.time as i64) {
            chrono::LocalResult::Ambiguous(a, _) => a,
            chrono::LocalResult::None => return Err("failed to create time".to_string()),
            chrono::LocalResult::Single(s) => s,
        };
        // Formats the combined date and time with the specified format string.
        let time: String = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        let transport = match Transport::from_str(&resor.transport) {
            Ok(s) => s,
            Err(_) => return Err("Transport is wrong".to_string()),
        };
        let from = Address::new(resor.from_id, resor.from_addres.clone());
        let by = match resor.by_id {
            Some(id) => match resor.by_addres.clone() {
                Some(address) => Some(Address::new(id, address)),
                None => None,
            },
            None => None,
        };
        //let by = Some(Address::new(resor.by_id, resor.by_addres.clone()));

        let to = Address::new(resor.to_id, resor.to_addres.clone());
        let notes = Some(String::new());
        let reservation_status = match resor.status.clone() {
            Some(status) => ReservationStatusEnum::from_str(&status)
                .ok()
                .and_then(|e| Some(ReservationStatus { status: e })),
            None => None,
        };
        let customer_info = CustomerInfo {
            phoneNumber: user.phone_number.clone(),
            notes: notes.clone(),
            hasReservationStatus: match reservation_status.is_some() {
                true => Some(true),
                false => None,
            },
            reservationStatus: reservation_status,
        };
        let departure = Departure::new(
            resor.time.to_string(),
            resor.transport.clone(),
            resor.company_name.clone(),
            resor.id.to_string(),
            time,
            resor.cancelleable,
            from.clone(),
            by.clone(),
            to.clone(),
            customer_info,
        )?;
        let attributes = Vec::new();
        let appliances = Vec::new();
        Ok(Self::new(
            resor.time.to_string(),
            user.name.clone(),
            user.card_nummer.to_string(),
            user.phone_number.to_string(),
            notes,
            resor.id.to_string(),
            vec![transport.clone()],
            transport,
            from,
            by,
            to,
            departure,
            resor.passagers,
            resor.child_passagers,
            attributes,
            appliances,
            resor.is_shared,
            resor.can_be_new_trip_template,
            resor.cancelleable,
        ))
    }
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: String,
        customerName: String,
        customerCardNumber: String,
        phoneNumber: String,
        notes: Option<String>,
        allocationId: String,
        transports: Vec<Transport>,
        transport: Transport,
        from: Address,
        by: Option<Address>,
        to: Address,
        departure: Departure,
        passengers: i32,
        childPassengers: i32,
        attributes: Vec<String>,
        appliances: Vec<String>,
        isShared: bool,
        canBeNewTripTemplate: bool,
        canBeCancelled: bool,
    ) -> Trips {
        Trips {
            id,
            customerName,
            customerCardNumber,
            phoneNumber,
            notes,
            allocationId,
            transport,
            transports,
            from,
            by,
            to,
            appliances,
            attributes,
            departure,
            canBeCancelled,
            canBeNewTripTemplate,
            childPassengers,
            passengers,
            isShared,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TripsRequest {
    group: String,
    sortOrder: String,
    filter: String,
    skip: i32,
    take: i32,
    remaining: Option<i32>,
    pub customerTransportReservation: Option<Vec<Trips>>,
}
impl TripsRequest {
    pub fn addTrips(
        mut self,
        user: &users::Model,
        resor: &Vec<resor::Model>,
    ) -> Result<TripsRequest, String> {
        let mut vec: Vec<Trips> = Vec::new();
        for resa in resor {
            vec.push(Trips::try_new(user, resa)?);
        }
        self.customerTransportReservation = Some(vec);
        Ok(self)
    }
    pub fn generate_query(&self) -> Select<db::resor::Entity> {
        use db::sea_orm::ColumnTrait;
        use db::sea_orm::QueryTrait;
        let mut query = resor::Entity::find();
        query= match self.filter.as_str(){
            "SharedTrip"=>query.filter(resor::Column::IsShared.eq(true)),
            "NotSharedTrip"=>query.filter(resor::Column::IsShared.eq(false)),
            "Taxi"=>query.filter(resor::Column::Transport.eq("Taxi")),
            "WheelChairTaxi"=>query.filter(resor::Column::Transport.eq("WheelChairTaxi")),
            _=>query,
        };
        query = match self.sortOrder.as_str() {
            "TimeDescending" => query.order_by_asc(resor::Column::Time),
            "TimeAscending" => query.order_by_desc(resor::Column::Time),
            "FromAddress"=> query.order_by_asc(resor::Column::FromAddres),
            "ToAddress"=> query.order_by_asc(resor::Column::ToAddres),
            _=>query.order_by_asc(resor::Column::Time),
        };
        let time = Utc::now().checked_sub_signed(Duration::hours(1)).expect("shoukd be able to add 1 hour").timestamp();
        query = match self.group.as_str() {
            "Historical" => query.filter(Condition::any().add(resor::Column::Time.gte(time))),
            _ => query.filter(Condition::any().add(resor::Column::Time.lte(time))),
        };
        query = query.offset(self.skip as u64).limit(self.take as u64);
        info!("{}", query.build(DbBackend::Sqlite).to_string());
        query
    }
}
