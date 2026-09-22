fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match ubnfc_front::cli::run(&args) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("ubnfc: {e}");
            std::process::exit(2);
        }
    }
}
