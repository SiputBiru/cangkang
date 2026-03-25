mod compiler;
mod error;
mod frontmatter;
mod fs;
mod html;
mod lexer;
mod parser;

fn main() {
    if let Err(e) = compiler::build_site() {
        eprintln!("\n❌ Build Failed!");
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
