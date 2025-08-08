fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xext");       // Required by some X11 functions
    println!("cargo:rustc-link-lib=Xrender");    // Sometimes needed for rendering support
}
