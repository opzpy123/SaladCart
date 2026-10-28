#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn run_selftest() -> i32 {
    let root = std::env::temp_dir().join(format!("saladcart-selftest-{}", std::process::id()));
    let result = saladcart_lib::paths::AppPaths::new(root).and_then(|paths| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| saladcart_lib::error::AppError::Message(error.to_string()))?;
        runtime.block_on(saladcart_lib::selftest::run(&paths))
    });
    let (code, text) = match result {
        Ok(report) => (0, report),
        Err(error) => (1, format!("selftest failed: {error}")),
    };
    let log = std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("saladcart-selftest.log");
    let _ = std::fs::write(&log, &text);
    eprintln!("{text}");
    code
}

fn main() {
    if std::env::args().any(|arg| arg == "--selftest") {
        std::process::exit(run_selftest());
    }
    saladcart_lib::run()
}
