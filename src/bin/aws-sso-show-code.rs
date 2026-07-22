use clap::Parser;

#[derive(Parser)]
struct Args {
    code: String,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("{}", args.code);
    }
    show_code::show(&args.code);
}

mod show_code {
    #[cfg(target_os = "macos")]
    mod platform {
        use std::ffi::CString;

        #[link(name = "macos_window", kind = "static")]
        #[link(name = "AppKit", kind = "framework")]
        #[link(name = "Foundation", kind = "framework")]
        #[link(name = "CoreFoundation", kind = "framework")]
        unsafe extern "C" {
            fn showSSOCode(
                code: *const std::ffi::c_char,
                on_close_requested: unsafe extern "C" fn(),
            );
            pub fn closeSSOCode();
        }

        unsafe extern "C" fn on_close_requested() {
            unsafe { closeSSOCode() };
        }

        pub fn show(code: &str) {
            let c = CString::new(code).expect("code contains null byte");
            unsafe { showSSOCode(c.as_ptr(), on_close_requested) };
        }
    }

    #[cfg(not(target_os = "macos"))]
    mod platform {
        pub fn show(code: &str) {
            eprintln!("AWS SSO code: {code}");
        }
    }

    pub fn show(code: &str) {
        platform::show(code);
    }
}
