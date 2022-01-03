use pb_jelly_gen::GenProtos;

fn main() -> std::io::Result<()> {
    GenProtos::builder()
        .out_path("../protos/gen")
        .src_path("../protos")
        .cleanup_out_path(true)
        .gen_protos();

    Ok(())
}
