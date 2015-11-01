extern crate hyper;

use self::hyper::Client;

use std::process::{Child, Command, Stdio};
use std::thread;
use std::io::Read;

struct Bomb(Child);

// Don't leak child processes!
impl Drop for Bomb {
    fn drop(&mut self) {
        match self.0.kill() {
            Ok(()) => {},
            Err(e) => println!("Leaking child process: {:?}", e),
        }

        if thread::panicking() {
            let mut s = String::new();
            let stdout = self.0.stdout.as_mut().unwrap();
            stdout.read_to_string(&mut s).unwrap();

            println!("Unparsed Stdout:\n{}", s);
        }
    }
}

pub fn read_url(url: &str) -> String {
    let mut res = Client::new()
                         .get(url)
                         .send()
                         .unwrap();

    let mut s = String::new();
    res.read_to_string(&mut s).unwrap();
    s
}

pub fn run_example<F>(name: &str, f: F)
where F: FnOnce(u16) {
    let command = format!("target/debug/examples/{}", name);
    let child = Command::new(&command)
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap();

    let mut bomb = Bomb(child);
    let port = parse_port(&mut bomb);

    f(port);
}

fn parse_port(&mut Bomb(ref mut process): &mut Bomb) -> u16 {
    // stdout doesn't implement BufRead... *shrug*
    let stdout = process.stdout.as_mut().unwrap();

    let mut line = String::new();

    for c in stdout.bytes().map(|b| b.unwrap() as char) {
        if c == '\n' { break }

        line.push(c)
    }

    let port = {
        let s = line.rsplitn(2, ':').next().unwrap();
        s.parse().expect("Failed to parse port")
    };

    println!("Parsed: port={} from {:?}", port, line);
    port
}
