use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::Regex;

use crate::Packages;
use crate::packages::{Dependency, RelVersionedPackageNum};

use rpkg::debversion::{self};

const KEYVAL_REGEX : &str = r"(?P<key>(\w|-)+): (?P<value>.+)";
const PKGNAME_AND_VERSION_REGEX : &str = r"(?P<pkg>(\w|\.|\+|-)+)( \((?P<op>(<|=|>)(<|=|>)?) (?P<ver>.*)\))?";

impl Packages {
    /// Loads packages and version numbers from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate value into the installed_debvers map with the parsed version number.
    pub fn parse_installed(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    // do something with ip

                    // match the line as key value pair
                    match kv_regexp.captures(&ip)
                    {
                        None => (),
                        Some(caps) => 
                        {
                            // parse line into key val
                            let (key, value) = (caps.name("key").unwrap().as_str(), caps.name("value").unwrap().as_str());
                            match key
                            {
                                // if key is package, insert name into hash
                                "Package" =>
                                {
                                    current_package_num = self.get_package_num_inserting(value);
                                }
                                // if key is version, map current package to version
                                "Version" =>
                                {
                                    let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                    self.installed_debvers.insert(current_package_num, debver);
                                }
                                _ => ()
                            }
                        }
                    }
                }
            }
        }
        println!("Packages installed: {}", self.installed_debvers.keys().len());
    }

    /// Loads packages, version numbers, dependencies, and md5sums from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate values into the dependencies, md5sum, and available_debvers maps.
    pub fn parse_packages(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        let pkgver_regexp = Regex::new(PKGNAME_AND_VERSION_REGEX).unwrap();

        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    // do more things with ip

                    // match the line as key value pair
                    match kv_regexp.captures(&ip)
                    {
                        None => (),
                        Some(caps) => 
                        {
                            // parse line into key val
                            let (key, value) = (caps.name("key").unwrap().as_str(), caps.name("value").unwrap().as_str());
                            match key
                            {
                                // if key is package, insert name into hash
                                "Package" =>
                                {
                                    current_package_num = self.get_package_num_inserting(value);
                                }
                                // if key is version, map current package to version
                                "Version" =>
                                {
                                    let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                    self.available_debvers.insert(current_package_num, debver);
                                }
                                // map current package to md5
                                "MD5sum" =>
                                {
                                    self.md5sums.insert(current_package_num, value.to_string());
                                }
                                // map current package to vector of dependencies
                                "Depends" =>
                                {
                                    // make dependency list
                                    let dep_list = value.split(',');
                                    let mut dep_vec : Vec<Dependency> = Vec::new();
                                    for dep in dep_list {

                                        // for each dependency, make alternative list
                                        let alt_list = dep.split('|');
                                        let mut alt_vec : Vec<RelVersionedPackageNum> = Vec::new();
                                        for alt in alt_list {

                                            // store info for each alternative
                                            let mut alt_info = RelVersionedPackageNum {package_num: 0, rel_version: None};
                                            // match for package name, version, & associated operation 
                                            match pkgver_regexp.captures(alt)
                                            {
                                                None => (),
                                                Some(caps) =>
                                                {
                                                    // parse the alt info and add to struct
                                                    let (pkg, op, ver) = (caps.name("pkg").unwrap().as_str(), caps.name("op"), caps.name("ver"));
                                                    alt_info.package_num = self.get_package_num_inserting(pkg);

                                                    // some dependencies dont have version (assumes latest version when installed)
                                                    if !ver.is_none()
                                                    {
                                                        let op = op.unwrap().as_str().parse::<debversion::VersionRelation>().unwrap();
                                                        let ver = ver.unwrap().as_str();
                                                        alt_info.rel_version = Some((op, ver.to_string()));
                                                    }

                                                    // add alternatives
                                                    alt_vec.push(alt_info);
                                                }
                                            }
                                        }
                                        // add dependencies
                                        dep_vec.push(alt_vec);
                                    }
                                    self.dependencies.insert(current_package_num, dep_vec);
                                }
                                _ => ()
                            }
                        }
                    }
                }
            }
        }
        println!("Packages available: {}", self.available_debvers.keys().len());
    }
}


// standard template code downloaded from the Internet somewhere
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
