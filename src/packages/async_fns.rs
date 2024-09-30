use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use std::time::Duration;
use std::str;

use crate::Packages;

struct Collector(Box<String>);
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        (*self.0).push_str(str::from_utf8(&data.to_vec()).unwrap());
        Ok(data.len())
    }
}

const DEFAULT_SERVER : &str = "ece459.patricklam.ca:4590";
impl Drop for Packages {
    fn drop(&mut self) {
        self.execute()
    }
}

pub struct AsyncState {
    server : String,
    easys : Vec<Easy2Handle<Collector>>,
    multi : Multi,
}

impl AsyncState {
    pub fn new() -> AsyncState {
        AsyncState {
            server : String::from(DEFAULT_SERVER),
            easys : Vec::new(),
            multi : Multi::new(),
        }
    }
}

impl Packages {
    pub fn set_server(&mut self, new_server:&str) {
        self.async_state.server = String::from(new_server);
    }

    /// Retrieves the version number of pkg and calls enq_verify_with_version with that version number.
    pub fn enq_verify(&mut self, pkg:&str) {
        let version = self.get_available_debver(pkg);
        match version {
            None => { println!("Error: package {} not defined.", pkg); return },
            Some(v) => { 
                let vs = &v.to_string();
                self.enq_verify_with_version(pkg, vs); 
            }
        };
    }

    /// Enqueues a request for the provided version/package information. Stores any needed state to async_state so that execute() can handle the results and print out needed output.
    pub fn enq_verify_with_version(&mut self, pkg:&str, version:&str) {
        let url = format!("http://{}/rest/v1/checksums/{}/{}", self.async_state.server, pkg, urlencoding::encode(version));
        println!("queueing request {}", url);

        // add easy handles to multi
        let mut easy = Easy2::new(Collector(Box::new(String::new())));
        easy.url(&url).unwrap();
        easy.verbose(false).unwrap();
        let handle = self.async_state.multi.add2(easy).unwrap();
        self.async_state.easys.push(handle);
    }

    /// Asks curl to perform all enqueued requests. For requests that succeed with response code 200, compares received MD5sum with local MD5sum (perhaps stored earlier). For requests that fail with 400+, prints error message.
    pub fn execute(&mut self) {

        // execute all easy handles. wait until done or 30 secs of no events
        let multi = &self.async_state.multi;
        while multi.perform().unwrap() > 0
        {
            multi.wait(&mut [], Duration::from_secs(30)).unwrap();
        }

        // check each easy handle
        let easys = &mut self.async_state.easys;
        for mut eh in easys.drain(..)
        {
            // get package and version associated with handle
            let mut url : Vec<&str> = eh.effective_url().unwrap().unwrap().split('/').collect();
            let (ver, pkg) = (url.pop().unwrap().to_string(), url.pop().unwrap().to_string());

            // check response code. If OK, get md5 and compare.
            let mut handler_after = multi.remove2(eh).unwrap();
            let response_code = handler_after.response_code().unwrap();
            if response_code == 200
            {
                let md5 = &handler_after.get_ref().0;
                let same_md5sum = (**md5).eq(self.md5sums.get(self.package_name_to_num.get(&pkg).unwrap()).unwrap());
                println!("verifying {}, matches: {:?}", pkg, same_md5sum);
            }
            else if response_code >= 400 {
                println!("got error {} on request for package {} version {}", response_code, pkg, ver);
            }
        }
    }
}
