fn main() {
    tonic_build::compile_protos("proto/signatrust.proto").unwrap();
}
