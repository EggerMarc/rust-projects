fn main() -> Result<(), Box<dyn std::error::Error>>{
    tonic_build::compile_protos("proto/route_guide.proto")?;

    println!("Successfully build proton");
    Ok(())
}


