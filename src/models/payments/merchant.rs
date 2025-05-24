use std::str::FromStr;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::payments::payment::{FeeTypes, FullPayment, PaymentSides, PaymentStatuses, PaymentStatusesSlim, ToSQL};
use crate::models::payments::payment_proto;
use crate::models::payments::payment_proto::from_timestamp_to_chrono;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MerchantPayment {
    pub id: String,
    pub external_id: String,
    pub merchant_id: String,
    pub client_id: Option<String>,
    pub status: PaymentStatuses,
    pub payment_side: PaymentSides,
    pub currency: String,
    pub target_amount: Decimal,
    pub fiat_amount: Decimal,
    pub crypto_amount: Decimal,
    pub fee_type: FeeTypes,
    pub margin: Decimal,
    pub exchange_rate: Decimal,
    pub fiat_fee: Decimal,
    pub crypto_fee: Decimal,
    pub holder_name: String,
    pub holder_account: String,
    pub bank_name: String,
    pub method: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub deadline: NaiveDateTime,
}

impl ToSQL for MerchantPayment {
    fn sql() -> String {
        String::from("SELECT id, external_id, merchant_id, client_id, status,
        payment_side, currency, target_amount, fiat_amount, crypto_amount,
        fee_type, margin, exchange_rate, fiat_fee, crypto_fee, holder_name, holder_account,
        bank_name, method, created_at, updated_at, deadline
        FROM payments")
    }
}

impl From<tokio_postgres::Row> for MerchantPayment {
    fn from(row: tokio_postgres::Row) -> Self {
        Self{
            id: row.get("id"),
            external_id: row.get("external_id"),
            merchant_id: row.get("merchant_id"),
            client_id: row.get("client_id"),
            status: PaymentStatuses::from_str(row.get("status")).unwrap(),
            payment_side: PaymentSides::from_str(row.get("payment_side")).unwrap(),
            currency: row.get("currency"),
            target_amount: row.get("target_amount"),
            fiat_amount: row.get("fiat_amount"),
            crypto_amount: row.get("crypto_amount"),
            exchange_rate: row.get("exchange_rate"),
            fee_type: FeeTypes::from_str(row.get("fee_type")).unwrap(),
            margin: row.get("margin"),
            fiat_fee: row.get("fiat_fee"),
            crypto_fee: row.get("crypto_fee"),
            holder_name: row.get("holder_name"),
            holder_account: row.get("holder_account"),
            bank_name: row.get("bank_name"),
            method: row.get("method"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deadline: row.get("deadline"),
        }
    }
}

impl From<FullPayment> for MerchantPayment {
    fn from(payment: FullPayment) -> Self {
        Self{
            id: payment.id,
            external_id: payment.external_id,
            merchant_id: payment.merchant_id,
            client_id: payment.client_id,
            status: payment.status,
            payment_side: payment.payment_side,
            currency: payment.currency,
            target_amount: payment.target_amount,
            fiat_amount: payment.fiat_amount,
            crypto_amount: payment.crypto_amount,
            fee_type: payment.fee_type,
            margin: payment.margin,
            exchange_rate: payment.exchange_rate,
            fiat_fee: payment.fiat_fee,
            crypto_fee: payment.crypto_fee,
            holder_name: payment.holder_name,
            holder_account: payment.holder_account,
            bank_name: payment.bank_name,
            method: payment.method,
            created_at: payment.created_at,
            updated_at: payment.updated_at,
            deadline: payment.deadline,
        }
    }
}


impl From<payment_proto::PaymentProto> for MerchantPayment {
    fn from(payment_proto: payment_proto::PaymentProto) -> Self {
        Self{
            id: payment_proto.id,
            external_id: payment_proto.external_id,
            merchant_id: payment_proto.merchant_id,
            client_id: payment_proto.client_id,
            status: PaymentStatuses::from_str(payment_proto.status.as_str()).unwrap(),
            payment_side: PaymentSides::from_str(payment_proto.payment_side.as_str()).unwrap(),
            currency: payment_proto.currency,
            target_amount: Decimal::from_str(payment_proto.target_amount.as_str()).unwrap(),
            fiat_amount: Decimal::from_str(payment_proto.fiat_amount.as_str()).unwrap(),
            crypto_amount: Decimal::from_str(payment_proto.crypto_amount.as_str()).unwrap(),
            fee_type: FeeTypes::from_str(payment_proto.fee_type.as_str()).unwrap(),
            margin: Decimal::from_str(payment_proto.margin.as_str()).unwrap(),
            exchange_rate: Decimal::from_str(payment_proto.exchange_rate.as_str()).unwrap(),
            fiat_fee: Decimal::from_str(payment_proto.fiat_fee.as_str()).unwrap(),
            crypto_fee: Decimal::from_str(payment_proto.crypto_fee.as_str()).unwrap(),
            holder_name: payment_proto.holder_name,
            holder_account: payment_proto.holder_account,
            bank_name: payment_proto.bank_name,
            method: payment_proto.method,
            created_at: from_timestamp_to_chrono(payment_proto.created_at.unwrap()),
            updated_at: payment_proto.updated_at.map(from_timestamp_to_chrono),
            deadline: from_timestamp_to_chrono(payment_proto.deadline.unwrap()),
        }
    }
}


#[derive(Deserialize, Debug, Clone)]
pub struct GetMerchantPayments {
    pub id: Option<String>,
    pub client_id: Option<String>,
    pub status: Option<Vec<PaymentStatusesSlim>>,
    pub payment_side: Option<PaymentSides>,
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}