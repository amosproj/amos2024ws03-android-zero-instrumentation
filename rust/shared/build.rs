fn main() {
    tonic_build::configure()
        .compile_protos(&["../proto/counter.proto"], &["../proto"])
        .unwrap();
}