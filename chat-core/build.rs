fn main() {
    prost_build::compile_protos(&["schema/schema.proto"], &["schema/"]).expect("compiling schema");

    capnpc::CompilerCommand::new()
        .default_parent_module(vec!["event".into(), "capnp".into()])
        .src_prefix("schema")
        .file("schema/schema.capnp")
        .run()
        .expect("compiling schema");
}
