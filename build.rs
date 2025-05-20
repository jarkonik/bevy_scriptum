fn main() {
    // export RUBY_CONFIGURE_OPTS="--disable-shared --disable-install-doc --disable-install-rdoc"
    // ./configure --with-cc-opt=“-I/usr/local/ssl/include” –with-ld-opt=“-L/usr/local/ssl/lib -Wl,-Bstatic -lssl -lcrypto -Wl,-Bdynamic -ldl”

    println!("cargo:rustc-link-lib=z"); // TODO: if features Ruby
}
