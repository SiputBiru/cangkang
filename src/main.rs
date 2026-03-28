mod compiler;
mod error;
mod frontmatter;
mod fs;
mod html;
mod lexer;
mod logger;
mod parser;

fn main() {
    if let Err(e) = compiler::build_site() {
        log_error!("\n❌ Build Failed!");
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
