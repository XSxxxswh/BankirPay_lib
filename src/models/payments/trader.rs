use std::str::FromStr;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::payments::payment::{FullPayment, PaymentSides, PaymentStatuses, PaymentStatusesSlim, ToSQL};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TraderPayment {
    pub id: String,
    pub requisite_id: String,
    pub holder_account: String,
    pub bank_id: String,
    pub bank_name: String,
    pub method: String,
    pub status: PaymentStatuses,
    pub payment_side: PaymentSides,
    pub currency: String,
    pub margin: Decimal,
    pub exchange_rate: Decimal,
    pub fiat_amount: Decimal,
    pub crypto_amount: Decimal,
    pub crypto_earnings: Decimal,
    pub fiat_earnings: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deadline: NaiveDateTime,
}

impl ToSQL for TraderPayment {
    fn sql() -> String {
        "SELECT id, requisite_id, holder_account, bank_id, 
        bank_name, method, status, payment_side, currency, trader_margin, exchange_rate, fiat_amount, 
        trader_crypto_amount, trader_crypto_fee, trader_fiat_fee, created_at, updated_at, deadline
        FROM payments".to_owned()
    }
}

impl From<tokio_postgres::Row> for TraderPayment {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            requisite_id: row.get("requisite_id"),
            holder_account: row.get("holder_account"),
            bank_id: row.get("bank_id"),
            bank_name: row.get("bank_name"),
            method: row.get("method"),
            status: PaymentStatuses::from_str(row.get("status")).unwrap(),
            payment_side: PaymentSides::from_str(row.get("payment_side")).unwrap(),
            currency: row.get("currency"),
            margin: row.get("trader_margin"),
            exchange_rate: row.get("exchange_rate"),
            fiat_amount: row.get("fiat_amount"),
            crypto_amount: row.get("trader_crypto_amount"),
            crypto_earnings: row.get("trader_crypto_fee"),
            fiat_earnings: row.get("trader_fiat_fee"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deadline: row.get("deadline"),
        }
    }
}
impl From<FullPayment> for TraderPayment {
    fn from(it: FullPayment) -> Self {
        return Self{
            id: it.id,
            requisite_id: it.requisite_id,
            holder_account: it.holder_account,
            bank_id: it.bank_id,
            bank_name: it.bank_name,
            method: it.method,
            status: it.status,
            payment_side: it.payment_side,
            currency: it.currency,
            margin: it.margin,
            exchange_rate: it.exchange_rate,
            fiat_amount: it.fiat_amount,
            crypto_amount: it.crypto_amount,
            crypto_earnings: it.trader_crypto_fee,
            fiat_earnings: it.trader_fiat_fee,
            created_at: it.created_at,
            updated_at: it.updated_at,
            deadline: it.deadline,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct TraderPaymentBuilder {
    pub trader_fiat_fee: Decimal,
    pub trader_crypto_fee: Decimal,
    pub trader_crypto_amount: Decimal,
    pub trader_margin: Decimal,
}


#[derive(Debug, Default, Clone, Serialize)]
pub struct TraderPaymentSlim {
    pub id: String,
    pub holder_account: String,
    pub status: PaymentStatuses,
    pub payment_side: PaymentSides,
    pub currency: String,
    pub exchange_rate: Decimal,
    pub fiat_amount: Decimal,
    pub crypto_amount: Decimal,
    pub crypto_earnings: Decimal,
    pub created_at: NaiveDateTime,
    pub deadline: NaiveDateTime,
}

impl ToSQL for TraderPaymentSlim {
    fn sql() -> String {
        String::from("SELECT id, holder_account, status, payment_side, currency, exchange_rate, fiat_amount,
    trader_crypto_amount, trader_crypto_fee, created_at, deadline
    FROM payments")
    }
}
impl From<tokio_postgres::Row> for TraderPaymentSlim {
    fn from(row: tokio_postgres::Row) -> Self {
        Self{
            id: row.get("id"),
            holder_account: row.get("holder_account"),
            status: PaymentStatuses::from_str(row.get("status")).unwrap(),
            payment_side: PaymentSides::from_str(row.get("payment_side")).unwrap(),
            currency: row.get("currency"),
            exchange_rate: row.get("exchange_rate"),
            fiat_amount: row.get("fiat_amount"),
            crypto_amount: row.get("trader_crypto_amount"),
            crypto_earnings: row.get("trader_crypto_fee"),
            created_at: row.get("created_at"),
            deadline: row.get("deadline"),
        }
    }
}


#[derive(Debug,Deserialize, Clone)]
pub struct GetPaymentsTrader {
    pub id: Option<String>,
    pub status: Option<Vec<PaymentStatusesSlim>>,
    pub bank_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}