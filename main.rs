use clap::{App, Arg};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

fn main() -> anyhow::Result<()> {
    let arguments = App::new("cloudflare-2020-systems-engineering-assignment")
        .version("1.0")
        .about("Tool for making HTTP/1.1 requests and measuring statistics about them.")
        .author("JMS55")
        .arg(
            Arg::with_name("url")
                .long("url")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("number_of_requests")
                .long("profile")
                .takes_value(true),
        )
        .get_matches();

    let url = Url::parse(arguments.value_of("url").unwrap())?;
    let host = url
        .host_str()
        .ok_or(anyhow::Error::msg("URL has no host."))?;
    let path = url.path();
    match arguments.value_of("number_of_requests") {
        None => {
            let response_info = make_request(host, path)?;
            println!("{}", String::from_utf8_lossy(&response_info.response));
        }
        Some(i) => {
            let number_of_requests = i.parse::<usize>()?;

            let mut times = Vec::with_capacity(number_of_requests);
            let mut error_codes = Vec::with_capacity(number_of_requests);
            let mut sizes = Vec::with_capacity(number_of_requests);
            for _ in 0..number_of_requests {
                let response_info = make_request(host, path)?;
                times.push(response_info.time);
                if response_info.code / 100 != 2 {
                    error_codes.push(response_info.code);
                }
                sizes.push(response_info.response.len());
                thread::sleep(Duration::from_millis(500));
            }
            times.sort_unstable();

            println!("Number of requests: {}.", number_of_requests);
            println!(
                "Fastest time: {:?}.",
                times
                    .iter()
                    .fold(Duration::from_secs(u64::MAX), |a, b| a.min(*b))
            );
            println!(
                "Slowest time: {:?}.",
                times.iter().fold(Duration::from_secs(0), |a, b| a.max(*b))
            );
            println!(
                "Mean time: {:?}.",
                times.iter().fold(Duration::from_secs(0), |a, b| a + *b) / times.len() as u32
            );
            println!("Median time: {:?}.", times.get(times.len() / 2).unwrap());
            println!(
                "Success percentage: {}.",
                (number_of_requests - error_codes.len()) / number_of_requests
            );
            println!("Error codes: {:?}.", error_codes);
            println!(
                "Smallest response size: {} bytes.",
                sizes.iter().fold(usize::MAX, |a, b| a.min(*b))
            );
            println!(
                "Largest response size: {} bytes.",
                sizes.iter().fold(0, |a, b| a.max(*b))
            );
        }
    };

    Ok(())
}

fn make_request(host: &str, path: &str) -> anyhow::Result<ResponseInfo> {
    let start = Instant::now();

    let mut response = Vec::new();
    let mut stream = TcpStream::connect(format!("{}:80", host))?;
    write!(stream, "GET {} HTTP/1.1\r\n", path)?;
    write!(stream, "Host: {}\r\n", host)?;
    write!(stream, "Connection: close\r\n\r\n")?;
    stream.read_to_end(&mut response)?;

    let mut code = 0;
    let text = String::from_utf8_lossy(&response);
    for line in text.lines() {
        if line.contains("HTTP/1.1") {
            let line = line.split(" ");
            code = line.skip(1).next().unwrap().parse::<u32>()?;
            break;
        }
    }

    Ok(ResponseInfo {
        response,
        code,
        time: start.elapsed(),
    })
}

struct ResponseInfo {
    response: Vec<u8>,
    code: u32,
    time: Duration,
}