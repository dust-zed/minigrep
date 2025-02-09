use anstream::println;
use anstyle::{Color, RgbColor};
use clap::Parser;
use command::Args;
use ignore::{types::TypesBuilder, WalkBuilder};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::io::Read;
use std::sync::Arc;
use std::{error::Error, path::Path};
use tokio::time::Instant;
mod command;

lazy_static! {
    static ref GREEN: anstyle::Style = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 255, 0))));
    static ref BLUE: anstyle::Style = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 0, 255))));
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let now = Instant::now();
    run(args).await?;
    let elapsed = now.elapsed();
    println!("查找总共耗时: {}ms", elapsed.as_millis());
    Ok(())
}

async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let Args {
        path,
        name,
        content,
        max_depth,
        threads,
        ignore_case,
    } = args;
    let mut types_builder = TypesBuilder::new();
    // 添加文件名的glob模式匹配
    if let Some(name) = name {
        types_builder.add("test", &name).unwrap();
        types_builder.select("all");
    };
    let types = types_builder.build().unwrap();
    //是否并行匹配
    let thread_count = threads.map_or(0, |threads| threads);
    let parallel = if thread_count > 1 { true } else { false };
    if parallel {
        let walker = WalkBuilder::new(path)
            .types(types)
            .threads(thread_count)
            .build_parallel();
        walker.run(|| {
            let regex_opt = content.as_ref().map(|c| Arc::new(build_regex(c).unwrap()));
            Box::new(move |result| {
                use ignore::WalkState::*;
                //println!("{} thread id : {:?}",tx.capacity(), thread::current().id());
                match result {
                    Ok(entry) => {
                        if let Some(ref regex) = regex_opt {
                            let _ = search_content_from_file(&regex, entry.path());
                        }
                        Continue
                    }
                    Err(_) => Skip,
                }
            })
        });
    } else {
        let walker = WalkBuilder::new(path).types(types).build();
        for result in walker {
            match result {
                Ok(entry) => {
                    let regex_opt = content.as_ref().map(|c| Arc::new(build_regex(c).unwrap()));
                    if let Some(ref regex) = regex_opt {
                        let _ = search_content_from_file(&regex, entry.path());
                    }

                }
                Err(_) => {}
            }
        }
    }
    Ok(())
}

fn build_regex(pattern: &str) -> Result<Regex, regex::Error> {
    RegexBuilder::new(pattern).build()
}

fn search_content_from_file(pattern: &Regex, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let cap = file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
    let mut rdr = std::io::BufReader::new(file);
    let mut buf = String::with_capacity(cap);
    rdr.read_to_string(&mut buf)?;
    for (line_num, line) in buf.lines().enumerate() {
        if pattern.is_match(&line.trim_end()) {
            println!("{}{}\t{}{}", *GREEN, line_num + 1, *BLUE, line);
        }
    }
    Ok(())
}
