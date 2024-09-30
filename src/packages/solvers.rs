use std::collections::VecDeque;

use rpkg::debversion::{self};

use crate::Packages;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C, 
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let mut dependency_set = vec![];

        // implement worklist
        let mut worklist : VecDeque<i32> = VecDeque::new();
        // push root package to worklist
        worklist.push_front(*self.get_package_num(package_name));

        // while alts left to be traversed
        // get alt dependencies until no more new ones are found
        while !worklist.is_empty()
        {
            let alt_num = worklist.pop_back().unwrap();
            let alt_deps = self.dependencies.get(&alt_num).unwrap();
            for alt_dep in alt_deps
            {
                let alt_dep_num = alt_dep.get(0).unwrap().package_num;

                // do not re-add dependencies already worked on
                if !dependency_set.contains(&alt_dep_num)
                {
                    worklist.push_front(alt_dep_num);
                    dependency_set.push(alt_dep_num);
                }
            }
        }

        return dependency_set;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let mut dependencies_to_add : Vec<i32> = vec![];

        // implement more sophisticated worklist
        let mut worklist : VecDeque<i32> = VecDeque::new();
        // push root package to worklist
        worklist.push_front(*self.get_package_num(package_name));

        // while alts left to be traversed
        // get alt dependencies until no more new ones are found
        while !worklist.is_empty()
        {
            let alt_num = worklist.pop_back().unwrap();
            let alt_deps = self.dependencies.get(&alt_num).unwrap();
            for alt_dep in alt_deps
            {
                let alt_dep_num = alt_dep.get(0).unwrap().package_num;

                // do not re-add dependencies already worked on
                if !dependencies_to_add.contains(&alt_dep_num)
                {
                    // filter out deps that are satisfied
                    if !self.dep_is_satisfied(alt_dep).is_none() { continue; }

                    let alt_choices = self.dep_satisfied_by_wrong_version(alt_dep);
                    if alt_choices.is_empty() 
                    { // no alts installed --------------------------
                        
                        // add highest version pkg to sets
                        let mut highest_pkg = alt_dep.first().unwrap().package_num;
                        let mut highest_ver = self.get_available_debver(self.get_package_name(highest_pkg)).unwrap();  // assumuing available version satisfies
                        for alt in alt_dep
                        {
                            // check if alt is higher version than current
                            let alt_ver = self.get_available_debver(self.get_package_name(alt.package_num)).unwrap();
                            if debversion::cmp_debversion_with_op(&debversion::VersionRelation::StrictlyGreater, &alt_ver, &highest_ver)
                            {
                                // update highest
                                highest_pkg = alt.package_num;
                                highest_ver = alt_ver;
                            }
                        }
                        worklist.push_front(highest_pkg);
                        dependencies_to_add.push(highest_pkg);
                    }
                    else 
                    { // alt(s) installed but wrong ver -------------
                        
                        let mut highest_pkg = self.get_package_num(alt_choices.first().unwrap());
                        let mut highest_ver = self.get_installed_debver(alt_choices.first().unwrap()).unwrap();
                        for alt in alt_choices
                        {
                            let alt_ver = self.get_installed_debver(alt).unwrap();
                            if debversion::cmp_debversion_with_op(&debversion::VersionRelation::StrictlyGreater, &alt_ver, &highest_ver)
                            {
                                highest_pkg = self.get_package_num(alt);
                                highest_ver = alt_ver;
                            }
                        }
                        worklist.push_front(*highest_pkg);
                        dependencies_to_add.push(*highest_pkg);
                    }
                }
            }
        }

        return dependencies_to_add;
    }
}
