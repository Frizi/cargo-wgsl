mod cli;
mod naga;
mod pp;
mod server;
mod wgsl_error;

fn main() {
    let last = std::env::args().last();

    if let Some(last) = last {
        if last == "--server" {
            server::run();
        } else {
            cli::run();
        }
    } else {
        cli::run();
    }
}
