fn main() {
    #[cfg(feature = "ruby")]
    {
        println!("cargo:rustc-link-arg=-rdynamic");
        println!("cargo:rustc-link-arg=-lz");
        println!("cargo:rustc-link-lib=z");
    }
}
