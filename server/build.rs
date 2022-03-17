use protobuf_codegen_pure::{Codegen, Customize};

fn main() {
    Codegen::new()
        .input("../protos/sptf.proto")
        .include("../protos")
        .out_dir("src/protos")
        .customize(Customize {
            carllerche_bytes_for_string: Some(true),
            carllerche_bytes_for_bytes: Some(true),
            ..Customize::default()
        })
        .run()
        .expect("Protobuf Codegen failed.");
}
