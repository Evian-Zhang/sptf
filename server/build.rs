use protobuf_codegen_pure::{Codegen, Customize};

fn main() {
    println!("cargo:rerun-if-changed=../protos/");
    Codegen::new()
        .input("../protos/sptf.proto")
        .include("../protos")
        .out_dir("src/protos")
        .customize(Customize {
            carllerche_bytes_for_string: Some(true),
            carllerche_bytes_for_bytes: Some(true),
            serde_derive: Some(true),
            gen_mod_rs: Some(true),
            ..Customize::default()
        })
        .run()
        .expect("Protobuf Codegen failed.");
}
