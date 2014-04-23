#![crate_id = "curl#0.1"]
#![comment = "Rust bindings for libcurl"]
#![license = "MIT"]

#![crate_type = "lib"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(globs)]

/*! 
Higher level safe abstractions for curl
# Example

```rust
let req = Request::new();

req.get("http://www.google.co.uk");
req.put("http://some-website.org", "random-file.html");
```
*/

extern crate libc;

use ffi::*;
use std::str;
use std::c_str::CString;
use std::io::{File, IoResult, IoError, OtherIoError};
use std::path::Path;

pub mod ffi;


struct UrlData {
    contents: Vec<char>
}

/// A Request abstracts over a CURL object
/// It allows you to set options and use common HTTP methods safely
pub struct Request {
    curl: *mut CURL
}

impl Request {
    /// Create a new Request, with default curl options
    pub fn new() -> Request {
        unsafe {
            Request { curl: curl_easy_init() }
        }
    }

    /// Sets an option on the underlying CURL object
    /// This method is very untype safe sorry, blame curl
    pub fn setOpt<X>(&self, opt: CURLoption, x: X) -> CURLcode {
        unsafe {
            curl_easy_setopt(self.curl, opt, x)
        }
    }

    /// curl_easy_perform the underlying CURL object
    /// You are not required to call this after any methods on this struct
    /// Do not call this unless you are sure of what options are currently set
    pub fn execute(&self) -> CURLcode {
        unsafe {
            curl_easy_perform(self.curl)
        }
    }

    /// Call HTTP GET on url and returns the result as a string
    pub fn get(&self, url: &str) -> IoResult<~str> {
        unsafe {
            self.setOpt(CURLOPT_URL, url);
            self.setOpt(CURLOPT_WRITEFUNCTION, write_data);

            let req = UrlData { contents: vec!() };
            self.setOpt(CURLOPT_WRITEDATA, &req);

            let res = self.execute();
            if res != CURLE_OK {
                return Err(curl_err_to_io_err(res));
            }
            
            let string: Vec<~str> = req.contents.iter().map(|&x| {
                str::from_char(x)
            }).collect();
            let string = string.concat();

            Ok(string)
        }
    }

    /// Download a file and save it to the file path specified
    pub fn download(&self, url: &str, file_path: &str) -> IoResult<()> {
        unsafe {
            let file = File::create(&Path::new(file_path));
            if file.is_err() {
                return Err(file.err().unwrap());
            }
            let file: File = file.unwrap();
            
            self.setOpt(CURLOPT_URL, url);
            self.setOpt(CURLOPT_WRITEFUNCTION, write_file);
            self.setOpt(CURLOPT_WRITEDATA, &file);

            let res = self.execute();
            if res != CURLE_OK {
                return Err(curl_err_to_io_err(res));
            }

            Ok(())
        }
    }

    /// Call HTTP POST on url, with the specified data
    /// data should be in the format "lang=rust&project=curl"
    pub fn post(&self, url: &str, data: &str) -> IoResult<()> {
        unsafe {
            self.setOpt(CURLOPT_URL, url);
            self.setOpt(CURLOPT_POSTFIELDS, data);

            let res = self.execute();
            if res != CURLE_OK {
                return Err(curl_err_to_io_err(res));
            }

            Ok(())
        }
    }

    /// Call HTTP PUT on url and upload the specified file name
    pub fn put(&self, url: &str, file_name: &str) -> IoResult<()> {
        unsafe {
            let file_info: *mut libc::stat = std::ptr::mut_null();
            let file_name = cstr_array(file_name);
            let file = libc::fopen(file_name, cstr_array("rb"));
            libc::stat(file_name, file_info);

            self.setOpt(CURLOPT_UPLOAD, 1);
            self.setOpt(CURLOPT_PUT, 1);
            self.setOpt(CURLOPT_URL, url);
            self.setOpt(CURLOPT_READDATA, file);
            
            let size: curl_off_t = std::cast::transmute((*file_info).st_size);
            self.setOpt(CURLOPT_INFILESIZE_LARGE, size);

            let res = self.execute();
            if res != CURLE_OK {
                return Err(curl_err_to_io_err(res));
            }

            Ok(())
        }
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        unsafe {
            curl_easy_cleanup(self.curl);
        }
    }
}

unsafe fn cstr_array(s: &str) -> *libc::c_char {
    s.to_c_str().unwrap()
}

unsafe fn curl_err_to_io_err(err: libc::c_uint) -> IoError {
    // Curl owns the string
    let desc = CString::new(curl_easy_strerror(err), false);
    let desc = desc.as_str().map(|x| x.to_owned());
    IoError {
        kind: OtherIoError,
        desc: "Curl has returned an error",
        detail: desc
    }
}

extern fn write_data(ptr: *libc::c_void, size: libc::size_t, 
                     nmemb: libc::size_t, data: *mut UrlData) -> libc::size_t {
    unsafe {
        let n: u64 = nmemb * size;
        for i in range(0, n) {
            let content = ptr.offset(i as int).to_option();

            // I think fail! is the wrong thing to do here, not sure
            // what else I could do though
            let content = match content {
                Some(x) => ((*x) as u8) as char,
                None    => fail!("Invalid memory")
            };

            match data.is_null() {
                false => (*data).contents.push(content),
                true  => fail!("Invalid memory")
            };
        }
    }
    nmemb * size
}

extern fn write_file(ptr: *libc::c_void, size: libc::size_t,
                     nmemb: libc::size_t, file: *mut File) -> libc::size_t {
    unsafe {
        let n = nmemb * size;
        for i in range(0, n) {
            let content = ptr.offset(i as int).to_option();

            let content = match content  {
                Some(x) => ((*x) as u8) as char,
                None    => fail!("Invalid memory")
            };

            match file.is_null() {
                // Uh, what do I do if this fails, fail! seems far to
                // extreme for simply not being able write to a file
                false => (*file).write_str(str::from_char(content)),
                true  => fail!("Invalid memory")
            };
        }
        n
    }
}

