fn main() {
    println!("cargo:rerun-if-changed=../model/migrations");
}
