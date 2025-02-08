use anstream::println;
use anstyle::{Color, RgbColor, Style};
use clap::error::ErrorKind;
use clap::Parser;
use command::Args;
use ignore::{types::TypesBuilder, WalkBuilder};
use regex::Regex;
use tokio::{join, task};
use std::sync::mpsc::channel;
use std::thread;
use std::{error::Error, path::Path};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::time::Instant;
mod command;
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
    let threads = threads.map_or(0, |threads| threads);
    let parallel = if threads > 1 { true } else { false };
    let (tx, rx) = channel();

    if parallel {
        let walker = WalkBuilder::new(path).types(types).threads(threads).build_parallel();
        walker.run(|| {
            let tx = tx.clone();
            Box::new(move |result | {
                use ignore::WalkState::*;
                //println!("thread id : {:?}", thread::current().id());
                match result {
                    Ok(entry) => {
                        tx.send(entry).unwrap();
                        Continue
                    }
                    Err(_) => Skip
                }
            })
        });
    } else {
        let walker = WalkBuilder::new(path).types(types).build();
        for result in walker {
            match result {
                Ok(entry) => {
                    tx.send(entry).unwrap();
                }
                Err(_) => {}
            }
        }
    }
    drop(tx);
    let walks = thread::spawn(move || {
        let mut vec = Vec::new();
        loop {
            match rx.recv() {
                Ok(entry) => {
                    vec.push(entry);
                }
                Err(_) => break,
            }
        }
        vec
    });
    let entries = walks.join().unwrap();
    for entry in entries {
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            continue;
        } else if metadata.is_file() {
            if let Some(ref content) = content {
                let regex = Regex::new(content).expect("build regex error");
                search_content(&regex, entry.path()).await?;
            }
        } else {
            continue;
        }
    }
    Ok(())
}

async fn search_content(pattern: &Regex, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path).await?;
    let mut read = BufReader::new(file);
    let mut buffer = String::new();
    let green = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 255, 0))));
    let blue = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 0, 255))));
    //行号
    let mut line_num = 0;
    loop {
        buffer.clear();
        line_num += 1;
        match read.read_line(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if pattern.is_match(&buffer.trim_end()) {
                    println!("{}{}\t{}{}", green, line_num, blue, buffer);
                }
                buffer.clear();
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}