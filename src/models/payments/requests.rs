use tokio_postgres::types::{ToSql, Type};
use crate::models::payments::merchant::GetMerchantPayments;
use crate::models::payments::payment::{GetPaymentRequestAdmin, PaymentSides};
use crate::models::payments::trader::GetPaymentsTrader;



#[derive(Debug)]
pub enum GetPaymentsRequest {
    Merchant((String, Option<GetMerchantPayments>)),
    Trader((String, Option<GetPaymentsTrader>)),
    Admin(Option<GetPaymentRequestAdmin>),
    
}

impl GetPaymentsRequest {
    // делает SQL строку из условий и массив из параметров
    pub fn to_sql(&self) -> (String, Vec<(&(dyn ToSql + Sync), Type)>) {
        let mut from_exists = false;
        let mut to_exists = false;
        match self {
            GetPaymentsRequest::Merchant((merchant_id, request)) => {
                let mut query = String::from(" WHERE merchant_id=$1");
                let mut query_conditions = Vec::<String>::with_capacity(5);
                let mut query_params: Vec<(&(dyn ToSql + Sync), Type)> = Vec::with_capacity(10);
                query_params.push((merchant_id, Type::VARCHAR));
                let mut param_index = 2;
                if let Some(request) = request {
                    if let Some(id) = request.id.as_ref() {
                        query_conditions.push(format!("id=${}", param_index));
                        query_params.push((id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(side) = request.payment_side.as_ref() {
                        query_conditions.push(format!("payment_side=${}", param_index));
                        match side {
                            PaymentSides::Buy => query_params.push((&"BUY", Type::VARCHAR)),
                            PaymentSides::Sell => query_params.push((&"SELL", Type::VARCHAR)),
                        }
                        param_index += 1;
                    }
                    if let Some(from) = request.from.as_ref() {
                        query_params.push((from, Type::TIMESTAMP));
                        query_conditions.push(format!("created_at>=${}", param_index));
                        param_index += 1;
                        from_exists = true;
                    }
                    if let Some(to) = request.to.as_ref() {
                        query_params.push((to, Type::TIMESTAMP));
                        query_conditions.push(format!("created_at<=${}", param_index));
                        param_index += 1;
                        to_exists = true;
                    }
                    if let Some(statuses) = request.status.as_ref() {
                        query_conditions.push(format!("status IN ({})", statuses.iter().map(|status| status.get_statuses_for_sql_query())
                            .collect::<Vec<_>>().join(", ")));
                    }
                    if !query_conditions.is_empty() {
                        query.push_str(" AND ");
                    }
                    query.push_str(query_conditions.join(" AND ").as_str());
                    if from_exists && to_exists {
                        if (request.to.unwrap() - request.from.unwrap()).num_days() > 30 {
                            let limit = request.limit.unwrap_or(100).min(100);
                            let offset = (request.page.unwrap_or(1) - 1) * limit;
                            query.push_str(format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset).as_str());
                        }else {
                            query.push_str(" ORDER BY created_at DESC");
                        }
                    }else {
                        let limit = request.limit.unwrap_or(100).min(100);
                        let offset = (request.page.unwrap_or(1) - 1) * limit;
                        query.push_str(format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset).as_str());
                    }

                }

                (query, query_params)
            },
            GetPaymentsRequest::Trader((trader_id, request)) => {
                let mut query = String::from(" WHERE trader_id=$1");
                let mut query_conditions = Vec::<String>::with_capacity(5);
                let mut query_params: Vec<(&(dyn ToSql + Sync), Type)> = Vec::with_capacity(10);
                query_params.push((trader_id, Type::VARCHAR));
                let mut param_index = 2;
                if let Some(request) = request {
                    if let Some(bank_id) = request.bank_id.as_ref() {
                        query_conditions.push(format!("bank_id=${}", param_index));
                        query_params.push((bank_id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(id) = request.id.as_ref() {
                        query_conditions.push(format!("id=${}", param_index));
                        query_params.push((id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(states) = request.status.as_ref() {
                        query_conditions.push(format!("status IN ({})", states.iter().map(|status| status.get_statuses_for_sql_query())
                            .collect::<Vec<_>>().join(", ")));
                    }
                    let limit = request.limit.unwrap_or(50).min(50);
                    let offset = (request.page.unwrap_or(1) - 1) * limit;
                    if !query_conditions.is_empty() {
                        query.push_str(" AND ");
                    }
                    query.push_str(query_conditions.join(" AND ").as_str());
                    query.push_str(format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset).as_str());
                }

                (query, query_params)
            }
            GetPaymentsRequest::Admin(request) => {
                let mut query_conditions = Vec::<String>::with_capacity(5);
                let mut query_params: Vec<(&(dyn ToSql + Sync), Type)> = Vec::with_capacity(10);
                let mut query = String::new();
                let mut param_index = 1;
                if let Some(request) = request {
                    if let Some(trader_id) = request.trader_id.as_ref() {
                        query_conditions.push(format!("trader_id=${}", param_index));
                        query_params.push((trader_id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(bank_id) = request.bank_id.as_ref() {
                        query_conditions.push(format!("bank_id=${}", param_index));
                        query_params.push((bank_id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(id) = request.id.as_ref() {
                        query_conditions.push(format!("id=${}", param_index));
                        query_params.push((id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(merchant_id) = request.merchant_id.as_ref() {
                        query_conditions.push(format!("merchant_id=${}", param_index));
                        query_params.push((merchant_id, Type::VARCHAR));
                    }
                    if let Some(client_id) = request.client_id.as_ref() {
                        query_conditions.push(format!("client_id=${}", param_index));
                        query_params.push((client_id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(requisite_id) = request.requisite_id.as_ref() {
                        query_conditions.push(format!("requisite_id=${}", param_index));
                        query_params.push((requisite_id, Type::VARCHAR));
                        param_index += 1;
                    }
                    if let Some(states) = request.statuses.as_ref() {
                        query_conditions.push(format!("status IN ({})", states.iter().map(|status| status.get_statuses_for_sql_query())
                            .collect::<Vec<_>>().join(", ")));
                    }
                    let limit = request.limit.unwrap_or(50).min(50);
                    let offset = (request.page.unwrap_or(1) - 1) * limit;
                    if !query_conditions.is_empty() {
                        query.push_str(" WHERE ");
                    }
                    query.push_str(query_conditions.join(" AND ").as_str());
                    query.push_str(format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset).as_str());
                }

                (query, query_params)
            }
        }
    }
    pub fn single_query<'a>(&'a self, mut sql_index: Option<&mut u32>) -> (String, Option<&'a str>) {
        // создает SQL строку условий без фильтров и с id юзера если оно есть 
        match self { 
            GetPaymentsRequest::Trader((id, _)) => {
                let index = match sql_index {
                    Some(ref mut i) => {
                        let current = **i;
                        **i += 1;
                        current
                    }
                    None => 1,
                };

                let query = format!(" WHERE trader_id=${}", index);
                (query, Some(id.as_str()))

            },
            GetPaymentsRequest::Merchant((id, _)) => {
                let index = match sql_index {
                    Some(ref mut i) => {
                        let current = **i;
                        **i += 1;
                        current
                    }
                    None => 1,
                };
                (format!(" WHERE merchant_id=${}", index), Some(id.as_str()))
            },
            _ => (" ".to_string(), None),
        }
    }
    pub fn get_requested_limit(self) -> usize {
        // возвращает запрошенный лимит
        match self { 
            GetPaymentsRequest::Merchant((_, request)) => {
                request.unwrap().limit.unwrap_or(50).min(50) as usize
            }
            GetPaymentsRequest::Trader((_, request)) => {
                request.unwrap().limit.unwrap_or(50).min(50) as usize
            },
            GetPaymentsRequest::Admin(req) => { 
                req.unwrap().limit.unwrap_or(50).min(50) as usize
            },
          
        }
    }
    
}