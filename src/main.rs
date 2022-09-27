use std::{io::BufRead};
use reqwest;
use tokio;
use argparse;
use colored::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Method {
    Get,
    Post,
    Put,
    Delete,
}

async fn request(url: String, method: Method) -> u16 {
    let client = reqwest::Client::new();

    match method {
        Method::Get => {
            let res = client.get(&url).send().await;
            match res {
                Ok(res) => res.status().as_u16(),
                Err(_) => 500,
            }
        }
        Method::Post => {
            let res = client.post(&url).send().await;
            match res {
                Ok(res) => res.status().as_u16(),
                Err(_) => 500,
            }
        }
        Method::Put => {
            let res = client.put(&url).send().await;
            match res {
                Ok(res) => res.status().as_u16(),
                Err(_) => 500,
            }
        }
        Method::Delete => {
            let res = client.delete(&url).send().await;
            match res {
                Ok(res) => res.status().as_u16(),
                Err(_) => 500,
            }
        }
    }
}

fn color_code_for_status(status: u16) -> String {
    match status {
        100..=199 => status.to_string().blue().to_string(),
        200..=299 => status.to_string().green().to_string(),
        300..=399 => status.to_string().yellow().to_string(),
        400..=499 => status.to_string().red().to_string(),
        500..=599 => status.to_string().red().to_string(),
        _ => status.to_string().white().to_string(),
    }
    .to_string()
}

fn load_directory_list(list: String) -> Result<Vec<String>, std::io::Error> {
    let mut urls = Vec::new();
    for line in std::io::BufReader::new(std::fs::File::open(list)?).lines() {
        let line = line?;
        if !line.starts_with("/") {
            urls.push(format!("/{}", line));
        } else {
            urls.push(line);
        }
    }
    Ok(urls)
}

fn print_menu(method: String, url: String, length_of_list: u32, verbose: bool) {
    println!(r#"
        ____        ________
       / __ \__  __/ __/ __/
      / /_/ / / / / /_/ /_  
     / _, _/ /_/ / __/ __/  
    /_/ |_|\__,_/_/ /_/"#);
    println!("\t\t\tv{}", VERSION);
    println!("───────────────────────────────────");
    println!("-> Method    : {}", method.to_uppercase());
    println!("-> URL       : {}", url);
    println!("-> List Size : {}", length_of_list);
    println!("-> Verbose   : {}", verbose);
    println!("───────────────────────────────────\n");
}

#[tokio::main]
async fn main() {
    let mut url = String::new();
    let mut directory_list = String::new();
    let mut method = String::new();
    let mut verbose = false;

    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Perform directory/file reconnaissance on a website");
        parser.refer(&mut url).add_option(&["-u", "--url"], argparse::Store, "URL to scan").required();
        parser.refer(&mut directory_list).add_option(&["-d", "--directory-list"], argparse::Store, "File containing list of directories to scan").required();
        parser.refer(&mut method).add_option(&["-m", "--method"], argparse::Store, "HTTP method to use").required();
        parser.refer(&mut verbose).add_option(&["-v", "--verbose"], argparse::StoreTrue, "Verbose output");
        parser.parse_args_or_exit();
    }

    if url.ends_with("/") {
        url.pop();
    }

    let urls = load_directory_list(directory_list);
    match urls {
        Ok(urls) => {
            let length_of_list = urls.len() as u32;
            print_menu(method.clone(), url.clone(), length_of_list, verbose);

            let mut futures = Vec::new();
            let longest_path = 50;
            for u in urls {
                let url = format!("{}{}", url.trim(), u.trim());
                let method = match method.as_str() {
                    "get" => Method::Get,
                    "post" => Method::Post,
                    "put" => Method::Put,
                    "delete" => Method::Delete,
                    _ => Method::Get,
                };
                futures.push(tokio::spawn(async move {
                    let status = request(url.clone(), method).await;
                    let status_colored = color_code_for_status(status);
                    let mut spaces: String = String::new();
                    if url.len() < longest_path {
                        spaces = " ".repeat(longest_path - url.len());
                    }
                    
                    if verbose {
                        println!("{}{} [Status: {}]", u, spaces, status_colored);
                    } else {
                        if status != 404 && status != 500 {
                            println!("{}{} [Status: {}]", u, spaces, status_colored);
                        }
                    }
                }));
            }
            for future in futures {
                future.await.unwrap();
            }
        }
        Err(e) => println!("error: {}", e),
    }
}