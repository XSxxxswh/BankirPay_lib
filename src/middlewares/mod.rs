use std::env;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use once_cell::sync::Lazy;
use tracing::error;
use crate::errors::LibError;
use crate::{models, use_case};
use crate::models::Claims;
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


