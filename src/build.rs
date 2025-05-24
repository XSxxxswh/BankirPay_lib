fn main() {
    tonic_build::compile_protos("src/proto/trader.proto").unwrap();
    tonic_build::compile_protos("src/proto/config.proto").unwrap();
    tonic_build::compile_protos("src/proto/merchant.proto").unwrap();
    tonic_build::compile_protos("src/proto/requisite.proto").unwrap();
    tonic_build::compile_protos("src/proto/notification.proto").unwrap();
    tonic_build::compile_protos("src/proto/bank.proto").unwrap();
    tonic_build::compile_protos("src/proto/payment.proto").unwrap();
    tonic_build::compile_protos("src/proto/exchange.proto").unwrap();
}