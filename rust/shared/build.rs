fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&[
            "../proto/counter.proto",
            "../proto/ziofa.proto",
            "../proto/config.proto"
        ], &["../proto"])
        .unwrap();
}