use rpkg::debversion::{self, DebianVersionNum};
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);
        // some sort of for loop...

        let pkg_num = self.get_package_num(package_name);
        let deps = self.dependencies.get(pkg_num).unwrap();

        // for all dependencies
        for dep in deps
        {
            // check if satisfied
            let alt_name = self.dep_is_satisfied(dep);
            println!("- dependency {:?}", self.dep2str(dep));
            if alt_name.is_none() {
                println!("-> not satisfied");
            }
            else {
                println!("+ {} satisfied by installed version {}", alt_name.unwrap(), self.get_installed_debver(alt_name.unwrap()).unwrap());
            }
        }
    }

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd:&Dependency) -> Option<&str> {
        // presumably you should loop on dd

        // for all alternatives in dependency...
        for alt in dd
        {
            let alt_name = self.get_package_name(alt.package_num);

            // is alternative installed?
            let inst_ver = self.get_installed_debver(alt_name);
            if inst_ver.is_none() { continue; } // no

            // is version satisfied?
            if !alt.rel_version.is_none()
            {
                let (op, alt_ver) = alt.rel_version.as_ref().unwrap();
                let alt_ver : DebianVersionNum = alt_ver.parse::<debversion::DebianVersionNum>().unwrap();

                if !debversion::cmp_debversion_with_op(op, inst_ver.unwrap(), &alt_ver) { continue; } // no
            }

            // satisfied!
            return Some(alt_name);
        }

        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd:&Dependency) -> Vec<&str> {
        assert! (self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        // another loop on dd

        for alt in dd
        {
            let alt_name = self.get_package_name(alt.package_num);
            
            // is alternative installed?
            let inst_ver = self.get_installed_debver(alt_name);
            if inst_ver.is_none() { continue; } // no

            // is version satisfied?
            if !alt.rel_version.is_none()
            {
                let (op, alt_ver) = alt.rel_version.as_ref().unwrap();
                let alt_ver : DebianVersionNum = alt_ver.parse::<debversion::DebianVersionNum>().unwrap();

                if !debversion::cmp_debversion_with_op(op, inst_ver.unwrap(), &alt_ver) { result.push(alt_name); } // wrong version
            }
        }
        return result;
    }
}

