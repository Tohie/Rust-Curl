extern crate curl;

fn main() {
    let req = curl::Request::new();
    println!("{}", req.get("http://www.google.co.uk"));
    println!("{}", req.get("http://www.facebook.co.uk"));
    req.download("http://www.google.co.uk", "google.html");
}
