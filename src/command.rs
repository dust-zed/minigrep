use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "dust zed", version = "0.1", about = "A simple command line app", long_about = None)]
pub struct Args {
    ///Path
    #[clap(short, long, default_value = ".")]
    pub path: String,
    ///name
    #[clap(short, long)]
    pub name: Option<String>,
    ///content
    #[clap(short, long)]
    pub content: Option<String>,
    ///max-depth
    #[clap(long)]
    pub max_depth: Option<u32>,
    ///threads
    #[clap(long)]
    pub threads: Option<u8>,
    ///ignore case
    #[clap(long)]
    pub ignore_case: Option<bool>
}