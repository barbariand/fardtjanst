#![allow(non_snake_case)] //beacuse needs serialzieble to json with CamelCaseNames :)
use chrono::{Utc, DateTime};

use serde::{Deserialize, Serialize};
use std::str::FromStr;
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
#[derive(Serialize, Deserialize, Debug,Clone,Copy)]
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
pub struct Address {
    id: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug,Clone, Copy,PartialEq, Eq)]
pub enum ReservationStatusEnum {
    ResaBesäld,
    BilPåväg,
    LetarEfterBil,
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
            "Letar efter bil"=> Ok(Self::LetarEfterBil),
            _ => Err(()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct ReservationStatus {
    status: ReservationStatusEnum,
}
fn is_none_or_empty(op: &Option<String>) -> bool {
    !match op {
        Some(s) => s.eq(""),
        None => false,
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
struct CustomerInfo {
    phoneNumber: String,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hasReservationStatus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reservationStatus: Option<ReservationStatus>,
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
    customerInfo: CustomerInfo,
}
impl Departure {
    pub fn get_time(&self)->Option<DateTime<Utc>>{
        DateTime::from_str(&self.departure).ok()
    }
    pub fn get_status(&self)->Option<ReservationStatusEnum>{
        self.customerInfo.reservationStatus.map(|s|s.status)
    }
}

#[derive(Serialize, Deserialize, Debug,Clone)]

pub struct Trips {
    pub id: String,
    pub customerName: String,
    customerCardNumber: String,
    phoneNumber: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    allocationId: String,
    transports: Vec<Transport>,
    transport: Transport,
    pub from: Address,
    by: Option<Address>,
    pub to: Address,
    departure: Departure,
    pub passengers: i32,
    pub childPassengers: i32,
    pub attributes: Vec<String>,
    pub appliances: Vec<String>,
    pub isShared: bool,
    pub canBeNewTripTemplate: bool,
    pub canBeCancelled: bool,
}
impl Trips{
    pub fn get_departure(&self)->Departure{
        self.departure.clone()
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
    pub fn new(group: String,
        sortOrder: String,
        filter: String,
        skip: i32,
        take: i32,
        remaining: Option<i32>)->Self{
            Self{
                group,
                sortOrder,
                filter,
                skip,
                take,
                remaining,
                customerTransportReservation:None,
            }
        }
}
