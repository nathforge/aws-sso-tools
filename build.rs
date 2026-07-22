fn main() {
    #[cfg(target_os = "macos")]
    cc::Build::new()
        .file("objc/aws_sso_show_code.m")
        .flag("-fobjc-arc")
        .compile("macos_window");
}
