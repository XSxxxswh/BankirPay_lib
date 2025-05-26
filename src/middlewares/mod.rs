use std::env;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use once_cell::sync::Lazy;
use tracing::error;
use crate::errors::LibError;
use crate::{models, use_case};
use crate::models::Claims;
use http_body_util::BodyExt;
pub static SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").expect("JWT_SECRET не задан")
});
pub async fn only_trader_middleware (
    State(state): State<Arc<models::AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let jwt = match req.headers().get("X-Token").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return LibError::Unauthorized.into_response(),
    };
    let claims = match verify_jwt(jwt) {
        Ok(claims) => claims,
        Err(e) => return e.into_response()
    };
    if claims.role != "trader" {
        return LibError::Forbidden.into_response();
    }
    let arc_claims = Arc::new(claims);
    if let Some(_) = arc_claims.impersonated_by { // если трейдер имперсонирован админом то не проверяем на блокировку
        req.extensions_mut().insert(arc_claims.clone());
        return next.run(req).await
    }
    match use_case::trader::check_trader_is_blocked(state, arc_claims.sub.as_str()).await {
        Ok(blocked) => {
            if blocked {
                return LibError::Unauthorized.into_response();
            }
        },
        Err(e) => return e.into_response()
    }
    req.extensions_mut().insert(arc_claims.clone());
    next.run(req).await
}


pub async fn only_merchant_middleware(
    State(state): State<Arc<models::AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let jwt = match req.headers().get("X-Token").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return LibError::Unauthorized.into_response(),
    };
    let claims = match verify_jwt(jwt) {
        Ok(claims) => claims,
        Err(e) => return e.into_response()
    };

    let arc_claims = Arc::new(claims);
    if arc_claims.role.to_lowercase() != "merchant" {
        return LibError::Forbidden.into_response();
    }
    if arc_claims.impersonated_by != None { // если мерчант имперсонирован то пропускаем без проверки на блок
        req.extensions_mut().insert(arc_claims.clone());
        return next.run(req).await
    }
    match use_case::merchant::check_merchant_is_blocked(state, arc_claims.sub.as_str()).await {
        Ok(a) if a => return LibError::Unauthorized.into_response(),
        Err(e) => match e {
            LibError::NotFound => return LibError::Forbidden.into_response(),
            LibError::MerchantNotFound => return LibError::Forbidden.into_response(),
            _ => return e.into_response(),
        },
        _ => ()
    }
    req.extensions_mut().insert(arc_claims.clone());
    next.run(req).await
}

pub async fn merchant_api_middleware(
    State(state): State<Arc<models::AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let (parts, body) = req.into_parts();
    let headers = parts.headers.clone();

    // 1. Получаем merchant_id из заголовка
    let merchant_id = match headers.get("X-Merchant-ID").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return LibError::Unauthorized.into_response(),
    };

    // 2. Получаем timestamp и проверяем формат
    let (timestamp, timestamp_str) = match headers.get("X-Timestamp").and_then(|h| h.to_str().ok()) {
        Some(ts) => match DateTime::parse_from_rfc3339(ts) {
            Ok(dt) => (dt, ts),
            Err(e) => {
                error!(err = e.to_string(), "Invalid timestamp");
                return LibError::Unauthorized.into_response();
            }
        },
        None => return LibError::Unauthorized.into_response(),
    };

    // 3. Проверка окна timestamp ±5 минут
    let now = Utc::now();
    let timestamp_utc = timestamp.with_timezone(&Utc);
    if (now - timestamp_utc).num_minutes().abs() > 5 {
        error!("Timestamp is outside allowed time window");
        return LibError::Unauthorized.into_response();
    }

    // 4. Получаем подпись из заголовка
    let sign = match headers.get("X-Signature").and_then(|h| h.to_str().ok()) {
        Some(sign) => sign,
        None => return LibError::Unauthorized.into_response(),
    };

    // 5. Собираем тело в bytes
    let body_bytes = match body.collect().await {
        Ok(agg) => agg.to_bytes(),
        Err(_) => return LibError::InternalError.into_response(),
    };

    // 6. Формируем строку для проверки подписи
    let raw_line = match parts.method {
        Method::GET => {
            format!("GET\n{}\n{}", merchant_id, timestamp_str)
        }
        Method::POST => {
            let body_str = String::from_utf8_lossy(&body_bytes);
            format!("POST\n{}\n{}\n{}", merchant_id, timestamp_str, body_str)
        }
        _ => return LibError::Unauthorized.into_response(),
    };

    // 7. Проверяем подпись (асинхронно, с твоей бизнес-логикой)
    match use_case::merchant::verify_signature(state.clone(), merchant_id, sign, raw_line.as_str()).await {
        Ok(valid) if valid => {
            let is_blocked = match use_case::merchant::check_merchant_is_blocked(state, merchant_id).await {
                Ok(is_blocked) => is_blocked,
                Err(e) => return e.into_response()
            };
            if is_blocked {
                return LibError::Forbidden.into_response();
            }
            // Всё хорошо, вставляем merchant_id в extensions
            let mut req = Request::from_parts(parts, body_bytes.into());
            req.extensions_mut().insert(merchant_id.to_string());
            next.run(req).await
        }
        Ok(_) => LibError::Unauthorized.into_response(),
        Err(e) => {
            error!(err = ?e, "Error verifying signature");
            e.into_response()
        }
    }
}



pub async fn only_admin_middleware (
    State(_): State<Arc<models::AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let jwt = match req.headers().get("X-Token").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return LibError::Unauthorized.into_response(),
    };
    let claims = match verify_jwt(jwt) {
        Ok(claims) => claims,
        Err(e) => return e.into_response()
    };

    let arc_claims = Arc::new(claims);
    if arc_claims.role.to_lowercase() != "admin"  {
        return LibError::Forbidden.into_response();
    }
    req.extensions_mut().insert(arc_claims.clone());
    next.run(req).await
}

pub async fn for_all_users_middleware (
    State(state): State<Arc<models::AuthState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let jwt = match req.headers().get("X-Token").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return LibError::Forbidden.into_response(),
    };
    let claims = match verify_jwt(jwt) {
        Ok(claims) => claims,
        Err(e) => return e.into_response()
    };

    let arc_claims = Arc::new(claims);
    if arc_claims.role.to_lowercase() == "trader" && arc_claims.impersonated_by == None { // если трейдер не админом то проверяем на блокировку
        match use_case::trader::check_trader_is_blocked(state, arc_claims.sub.as_str()).await {
            Ok(blocked) => {
                if blocked {
                    return LibError::Forbidden.into_response();
                }
            },
            Err(e) => return e.into_response()
        }
    }

    req.extensions_mut().insert(arc_claims.clone());
    next.run(req).await
}

fn verify_jwt(token: &str) -> Result<models::Claims, LibError> {
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(SECRET.as_bytes()), &Validation::default()).map_err(|e| {
        error!("error verify jwt, {}", e);
        LibError::Unauthorized
    }
    )?;
    Ok(token_data.claims)
}


