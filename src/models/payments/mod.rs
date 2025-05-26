pub mod payment;
pub mod trader;
pub mod merchant;
pub mod requests;


pub mod payment_proto {
    use chrono::NaiveDateTime;
    use crate::models::payments;

    tonic::include_proto!("payment");
    impl From<payments::payment::FullPayment> for PaymentProto {
        fn from(value: payments::payment::FullPayment) -> Self {
            return Self{
                id: value.id,
                external_id: value.external_id,
                merchant_id: value.merchant_id,
                client_id: value.client_id,
                trader_id: value.trader_id,
                requisite_id: value.requisite_id,
                bank_id: value.bank_id,
                status: value.status.to_string(),
                payment_side: value.payment_side.to_string(),
                currency: value.currency.to_string(),
                target_amount: value.target_amount.to_string(),
                fiat_amount: value.fiat_amount.to_string(),
                crypto_amount: value.crypto_amount.to_string(),
                trader_crypto_amount: value.trader_crypto_amount.to_string(),
                exchange_rate: value.exchange_rate.to_string(),
                fee_type: value.fee_type.to_string(),
                crypto_fee: value.crypto_fee.to_string(),
                fiat_fee: value.fiat_fee.to_string(),
                trader_crypto_fee: value.trader_crypto_fee.to_string(),
                trader_fiat_fee: value.trader_fiat_fee.to_string(),
                holder_name: value.holder_name,
                holder_account: value.holder_account,
                bank_name: value.bank_name,
                method: value.method,
                margin: value.margin.to_string(),
                trader_margin: value.trader_margin.to_string(),
                earnings: value.earnings.to_string(),
                created_at: Some(from_chrono_to_timestamp(value.created_at)),
                updated_at: value.updated_at.map(from_chrono_to_timestamp),
                deadline: Some(from_chrono_to_timestamp(value.deadline)),
                last_four: value.last_four,
                card_last_four: value.card_last_four,
                close_by: value.close_by,
            }
        }
    }

    pub fn from_timestamp_to_chrono(timestamp: prost_types::Timestamp) -> chrono::NaiveDateTime {
        return NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32);
    }
    pub fn from_chrono_to_timestamp(timestamp: chrono::NaiveDateTime) -> prost_types::Timestamp {
        prost_types::Timestamp{ seconds: timestamp.and_utc().timestamp(), nanos: timestamp.and_utc().timestamp_subsec_nanos() as i32 }
    }
}
