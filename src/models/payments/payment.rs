use std::fmt::{Display, Formatter};
use std::str::FromStr;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::payments::payment_proto;
use crate::models::payments::payment_proto::from_timestamp_to_chrono;

pub trait ToSQL {
    fn sql() -> String;
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FeeTypes {
    #[default]
    ChargeCustomer,
    ChargeMerchant
}

impl Display for FeeTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeeTypes::ChargeCustomer => f.write_str("charge_customer"),
            FeeTypes::ChargeMerchant => f.write_str("charge_merchant")
        }
    }
}

impl FromStr for FeeTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "charge_customer" => Ok(FeeTypes::ChargeCustomer),
            "charge_merchant" => Ok(FeeTypes::ChargeMerchant),
            _ => Err(format!("unknown fee_types {}", s))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum PaymentSides {
    #[default]
    Buy,
    Sell
}

impl Display for PaymentSides {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentSides::Buy => f.write_str("BUY"),
            PaymentSides::Sell => f.write_str("SELL")
        }
    }
}

impl FromStr for PaymentSides {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(PaymentSides::Buy),
            "SELL" => Ok(PaymentSides::Sell),
            _ => Err(format!("unknown payment sides {}", s))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatusesSlim {
    Completed,
    Processing,
    Frozen,
    Cancelled
}

impl PaymentStatusesSlim{
    pub fn get_statuses_for_sql_query(&self) -> String {
        match self {
            PaymentStatusesSlim::Completed => String::from("'COMPLETED'"),
            PaymentStatusesSlim::Processing => String::from("'PROCESSING', 'PAID'"),
            PaymentStatusesSlim::Frozen => String::from("'FROZEN'"),
            PaymentStatusesSlim::Cancelled => String::from("'CANCELLED_BY_TIMEOUT', 'CANCELLED_BY_ADMIN', \
            'CANCELLED_BY_TRADER', 'CANCELLED_BY_MERCHANT', 'CANCELLED_BY_CUSTOMER'")
        }
    }
}
impl Display for PaymentStatusesSlim {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            PaymentStatusesSlim::Completed => f.write_str("COMPLETED"),
            PaymentStatusesSlim::Processing => f.write_str("PROCESSING"),
            PaymentStatusesSlim::Frozen => f.write_str("FROZEN"),
            PaymentStatusesSlim::Cancelled => f.write_str("CANCELLED"),
        }
    }
}
impl FromStr for PaymentStatusesSlim {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() { 
            "COMPLETED" => Ok(PaymentStatusesSlim::Completed),
            "PROCESSING" => Ok(PaymentStatusesSlim::Processing),
            "FROZEN" => Ok(PaymentStatusesSlim::Frozen),
            "CANCELLED" => Ok(PaymentStatusesSlim::Cancelled),
            _ => Err(format!("unknown payment slim {}", s))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatuses {
    Pending,
    #[default]
    Unpaid,
    Paid,
    Completed,
    CancelledByTimeout,
    CancelledByMerchant,
    CancelledByCustomer,
    CancelledByAdmin,
    CancelledByTrader,
    Processing,
    Queued,
    Frozen
}

impl PaymentStatuses {
    pub fn is_success(&self) -> bool {
        matches!(self,PaymentStatuses::Completed)
    }
    pub fn is_cancelled(&self) -> bool {
        matches!(self,
            PaymentStatuses::CancelledByTimeout
        | PaymentStatuses::CancelledByMerchant
        | PaymentStatuses::CancelledByCustomer
        | PaymentStatuses::CancelledByAdmin
        | PaymentStatuses::CancelledByTrader)
    }
    pub fn is_final(&self) -> bool {
        matches!(self,PaymentStatuses::Completed
        | PaymentStatuses::CancelledByTimeout
        | PaymentStatuses::CancelledByMerchant
        | PaymentStatuses::CancelledByCustomer
        | PaymentStatuses::CancelledByAdmin
        | PaymentStatuses::CancelledByTrader)
    }
}



impl Display for PaymentStatuses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatuses::Unpaid => write!(f, "UNPAID"),
            PaymentStatuses::Paid => write!(f, "PAID"),
            PaymentStatuses::Completed => write!(f, "COMPLETED"),
            PaymentStatuses::CancelledByTimeout => write!(f, "CANCELLED_BY_TIMEOUT"),
            PaymentStatuses::CancelledByAdmin => write!(f, "CANCELLED_BY_ADMIN"),
            PaymentStatuses::CancelledByTrader => write!(f, "CANCELLED_BY_TRADER"),
            PaymentStatuses::CancelledByMerchant => write!(f, "CANCELLED_BY_MERCHANT"),
            PaymentStatuses::CancelledByCustomer => write!(f, "CANCELLED_BY_CUSTOMER"),
            PaymentStatuses::Processing => write!(f, "PROCESSING"),
            PaymentStatuses::Queued => write!(f, "QUEUED"),
            PaymentStatuses::Pending => write!(f, "PENDING"),
            PaymentStatuses::Frozen => write!(f, "FROZEN")
        }
    }
}
impl FromStr for PaymentStatuses {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNPAID" => Ok(PaymentStatuses::Unpaid),
            "PAID" => Ok(PaymentStatuses::Paid),
            "COMPLETED" => Ok(PaymentStatuses::Completed),
            "CANCELLED_BY_TIMEOUT" => Ok(PaymentStatuses::CancelledByTimeout),
            "CANCELLED_BY_ADMIN" => Ok(PaymentStatuses::CancelledByAdmin),
            "CANCELLED_BY_TRADER" => Ok(PaymentStatuses::CancelledByTrader),
            "CANCELLED_BY_MERCHANT" => Ok(PaymentStatuses::CancelledByMerchant),
            "CANCELLED_BY_CUSTOMER" => Ok(PaymentStatuses::CancelledByCustomer),
            "PROCESSING" => Ok(PaymentStatuses::Processing),
            "QUEUED" => Ok(PaymentStatuses::Queued),
            "PENDING" => Ok(PaymentStatuses::Pending),
            "FROZEN" => Ok(PaymentStatuses::Frozen),
            _ => Err(format!("unknown payment: {}", s)),
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct FullPayment {
    pub id: String,
    pub external_id: String,
    pub merchant_id: String,
    pub client_id: Option<String>,
    pub trader_id: String,
    pub requisite_id: String,
    pub bank_id: String,
    pub status: PaymentStatuses,
    pub payment_side: PaymentSides,
    pub currency: String,
    pub target_amount: Decimal,
    pub fiat_amount: Decimal,
    pub crypto_amount: Decimal,
    pub trader_crypto_amount: Decimal,
    pub exchange_rate: Decimal,
    pub fee_type: FeeTypes,
    pub crypto_fee: Decimal,
    pub fiat_fee: Decimal,
    pub trader_crypto_fee: Decimal,
    pub trader_fiat_fee: Decimal,
    pub holder_name: String,
    pub holder_account: String,
    pub bank_name: String,
    pub method: String,
    pub margin: Decimal,
    pub trader_margin: Decimal,
    pub earnings: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deadline: NaiveDateTime,
    pub last_four : String,
    pub card_last_four: String,
    pub close_by: Option<String>,
}


impl ToSQL for FullPayment {
    fn sql() -> String {
        "SELECT * FROM payments".to_owned()
    }
}

impl From<tokio_postgres::Row> for FullPayment {
    fn from(row: tokio_postgres::Row) -> Self {
        Self{
            id: row.get("id"),
            external_id: row.get("external_id"),
            merchant_id: row.get("merchant_id"),
            client_id: None,
            trader_id: row.get("trader_id"),
            requisite_id: row.get("requisite_id"),
            bank_id: row.get("bank_id"),
            status: PaymentStatuses::from_str(row.get::<_, String>("status").as_str()).unwrap(),
            payment_side: PaymentSides::from_str(row.get("payment_side")).unwrap(),
            currency: row.get("currency"),
            target_amount: row.get("target_amount"),
            fiat_amount: row.get("fiat_amount"),
            crypto_amount: row.get("crypto_amount"),
            trader_crypto_amount: row.get("trader_crypto_amount"),
            exchange_rate: row.get("exchange_rate"),
            fee_type: FeeTypes::from_str(row.get::<_, String>("fee_type").as_str()).unwrap(),
            crypto_fee: row.get("crypto_fee"),
            fiat_fee: row.get("fiat_fee"),
            trader_crypto_fee: row.get("trader_crypto_fee"),
            trader_fiat_fee: row.get("trader_fiat_fee"),
            holder_name: row.get("holder_name"),
            holder_account: row.get("holder_account"),
            bank_name: row.get("bank_name"),
            method: row.get("method"),
            margin: row.get("margin"),
            trader_margin: row.get("trader_margin"),
            earnings: row.get("earnings"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deadline: row.get("deadline"),
            last_four: row.get("last_four"),
            card_last_four: row.get("card_last_four"),
            close_by: row.get("close_by"),
        }
    }
}


impl From<&tokio_postgres::Row> for FullPayment {
    fn from(row: &tokio_postgres::Row) -> Self {
        Self{
            id: row.get("id"),
            external_id: row.get("external_id"),
            merchant_id: row.get("merchant_id"),
            client_id: None,
            trader_id: row.get("trader_id"),
            requisite_id: row.get("requisite_id"),
            bank_id: row.get("bank_id"),
            status: PaymentStatuses::from_str(row.get::<_, String>("status").as_str()).unwrap(),
            payment_side: PaymentSides::from_str(row.get("payment_side")).unwrap(),
            currency: row.get("currency"),
            target_amount: row.get("target_amount"),
            fiat_amount: row.get("fiat_amount"),
            crypto_amount: row.get("crypto_amount"),
            trader_crypto_amount: row.get("trader_crypto_amount"),
            exchange_rate: row.get("exchange_rate"),
            fee_type: FeeTypes::from_str(row.get::<_, String>("fee_type").as_str()).unwrap(),
            crypto_fee: row.get("crypto_fee"),
            fiat_fee: row.get("fiat_fee"),
            trader_crypto_fee: row.get("trader_crypto_fee"),
            trader_fiat_fee: row.get("trader_fiat_fee"),
            holder_name: row.get("holder_name"),
            holder_account: row.get("holder_account"),
            bank_name: row.get("bank_name"),
            method: row.get("method"),
            margin: row.get("margin"),
            trader_margin: row.get("trader_margin"),
            earnings: row.get("earnings"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deadline: row.get("deadline"),
            last_four: row.get("last_four"),
            card_last_four: row.get("card_last_four"),
            close_by: row.get("close_by"),
        }
    }
}
impl From<payment_proto::PaymentProto> for FullPayment {
    fn from(value: payment_proto::PaymentProto) -> Self {
        Self{
            id: value.id,
            external_id: value.external_id,
            merchant_id: value.merchant_id,
            client_id: value.client_id,
            trader_id: value.trader_id,
            requisite_id: value.requisite_id,
            bank_id: value.bank_id,
            status: PaymentStatuses::from_str(value.status.as_str()).unwrap(),
            payment_side: PaymentSides::from_str(value.payment_side.as_str()).unwrap(),
            currency: value.currency,
            target_amount: Decimal::from_str(value.target_amount.as_str()).unwrap(),
            fiat_amount: Decimal::from_str(value.fiat_amount.as_str()).unwrap(),
            crypto_amount: Decimal::from_str(value.crypto_amount.as_str()).unwrap(),
            trader_crypto_amount: Decimal::from_str(value.trader_crypto_amount.as_str()).unwrap(),
            exchange_rate: Decimal::from_str(value.exchange_rate.as_str()).unwrap(),
            fee_type: FeeTypes::from_str(value.fee_type.as_str()).unwrap(),
            crypto_fee: Decimal::from_str(value.crypto_fee.as_str()).unwrap(),
            fiat_fee: Decimal::from_str(value.fiat_fee.as_str()).unwrap(),
            trader_crypto_fee: Decimal::from_str(value.trader_crypto_fee.as_str()).unwrap(),
            trader_fiat_fee: Decimal::from_str(value.trader_fiat_fee.as_str()).unwrap(),
            holder_name: value.holder_name,
            holder_account: value.holder_account,
            bank_name: value.bank_name,
            method: value.method,
            margin: Decimal::from_str(value.margin.as_str()).unwrap(),
            trader_margin: Decimal::from_str(value.trader_margin.as_str()).unwrap(),
            earnings: Decimal::from_str(value.earnings.as_str()).unwrap(),
            created_at: from_timestamp_to_chrono(value.created_at.unwrap()),
            updated_at: value.updated_at.map(from_timestamp_to_chrono),
            deadline: from_timestamp_to_chrono(value.deadline.unwrap()),
            last_four: value.last_four,
            card_last_four: value.card_last_four,
            close_by: value.close_by,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewPaymentRequest {
    pub external_id: String,
    pub client_id: Option<String>,
    pub target_amount: Decimal,
    pub currency: String,
    pub currency_type: String,
    pub side: PaymentSides,
    pub fee_type: FeeTypes,
    pub method_id: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct GetPaymentRequestAdmin {
    pub id: Option<String>,
    pub client_id: Option<String>,
    pub external_id: Option<String>,
    pub bank_id: Option<String>,
    pub requisite_id: Option<String>,
    pub merchant_id: Option<String>,
    pub trader_id: Option<String>,
    pub statuses: Option<Vec<PaymentStatusesSlim>>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

