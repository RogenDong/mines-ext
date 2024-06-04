
fn main() {
    println!("cargo:rustc-env=JNI_JPATH=me.dong.mines.mines.MinesJNI");
    println!("cargo:rerun-if-changed=src/lib.rs");
}