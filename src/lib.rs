pub mod models;
mod build;
pub mod middlewares;
pub mod errors;
mod repository;
mod use_case;
pub mod services;

pub mod device_proto {
    tonic::include_proto!("device");
}

pub mod merchant_proto {
    tonic::include_proto!("merchant");
}
pub mod requisites_proto {
    tonic::include_proto!("requisites");
}

pub mod trader_proto {
    tonic::include_proto!("trader");
}

pub mod bank_proto {
    tonic::include_proto!("banks");
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
