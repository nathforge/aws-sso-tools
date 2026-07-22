use std::ffi::CString;
use std::os::unix::io::FromRawFd;
use std::process::{Command, Stdio};

fn main() {
    // Docker credential helpers communicate with Docker over stdout, so we
    // redirect aws-sso-maybe-login's stdout to stderr to avoid interference.
    let stderr_dup = unsafe { libc::dup(libc::STDERR_FILENO) };
    if stderr_dup == -1 {
        eprintln!("error: dup(2) failed: {}", std::io::Error::last_os_error());
        std::process::exit(1);
    }
    let status = Command::new(aws_sso_tools::sibling_bin("aws-sso-maybe-login"))
        .stdin(Stdio::inherit())
        .stdout(unsafe { Stdio::from_raw_fd(stderr_dup) })
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to run aws-sso-maybe-login: {e}");
            std::process::exit(1);
        });

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    let prog = CString::new("docker-credential-ecr-login").unwrap();
    let mut cargs = vec![prog.clone()];
    cargs.extend(std::env::args().skip(1).map(|a| CString::new(a).unwrap()));

    match nix::unistd::execvp(&prog, &cargs) {
        Ok(_) => unreachable!(),
        Err(e) => {
            eprintln!("error: failed to exec docker-credential-ecr-login: {e}");
            std::process::exit(1);
        }
    }
}
