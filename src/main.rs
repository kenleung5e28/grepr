fn main() {
    if let Err(e) = grepr::get_args().and_then(grep::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
