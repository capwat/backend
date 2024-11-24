fn main() {
    if let Ok(value) = std::env::var("PROFILE") {
        if value == "release" {
            println!("cargo:rustc-cfg=release");
        }
    } else {
        println!("cargo:error=failed to get build profile");
    }
}
