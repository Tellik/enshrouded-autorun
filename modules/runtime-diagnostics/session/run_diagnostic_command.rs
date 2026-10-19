use std::{env, path::PathBuf};

pub fn run_module() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    let arg = |name: &str| {
        args.windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].as_str())
    };
    let root = PathBuf::from(arg("--root").ok_or("missing --root installation directory")?);
    let job_id = arg("--job");

    if args.iter().any(|value| value == "--status") {
        println!("{}", crate::status(&root));
        return Ok(());
    }

    for action in ["start", "stop", "snapshot"] {
        if args.iter().any(|value| value == &format!("--{action}")) {
            crate::request_job(&root, action, job_id)?;
            println!("Diagnostic job request accepted; the running game loader processes it");
            return Ok(());
        }
    }

    Err("choose --start, --snapshot, --stop, or --status, optionally with --job <id>".into())
}
