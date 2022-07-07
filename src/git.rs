use git2::{IndexAddOption, Repository, Oid, Config, Branches};
use std::path::Path;

pub struct Git {
    repository: Repository
}

impl Git {

    pub fn new(folder: &str) -> Self {
        let repo = match Repository::init(folder) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to init: {}", e),
        };
        Self {repository: repo}
    }

    pub fn config(&self, user_name: &str, user_email: &str)
                  -> Result <Config, git2::Error>{

        let mut conf = Config::new().unwrap();
        let path_str = [&self.repository.path().to_str().unwrap(),
                        "config"].join("/");
        conf.add_file(Path::new(&path_str),
                      git2::ConfigLevel::Local,
                      true)?;
        conf.set_str("user.name", user_name)?;
        conf.set_str("user.email", user_email)?;
        Ok(conf)
    }

    pub fn add_all(&self) -> Result<Oid, git2::Error>{

        let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            if path.to_str().unwrap().ends_with("/") {
                1
            }else{
                let status = self.repository.status_file(path).unwrap();
                if status.contains(git2::Status::WT_MODIFIED)
                    || status.contains(git2::Status::WT_NEW)
                    || status.contains(git2::Status::IGNORED)
                    || status.contains(git2::Status::WT_DELETED){
                        0
                    }else {
                        1
                    }
            }
        };

        let mut index = self.repository.index()
            .expect("Git: cannot get the Index file.");
        index.add_all(["*"].iter(),
                      IndexAddOption::FORCE,
                      Some(cb as &mut git2::IndexMatchedPath))
            .expect("Git: failed to add -f *");
        index.write().expect("Git: failed to write modification.");
        index.write_tree()
    }

    pub fn commit(&self, msg: &str, tree_id: Oid) -> Result<Oid, git2::Error>  {
        let signature = self.repository.signature().unwrap();
        let tree = self.repository.find_tree(tree_id).unwrap();
        let head = self.repository.head();
        let ret: Result<Oid, git2::Error>;
        match head {
            Ok(h) => {
                ret = self.repository.commit(Some("HEAD"),
                                             &signature,
                                             &signature,
                                             msg,
                                             &tree,
                                             &[&h.peel_to_commit().unwrap()]
                );
            }
            Err(_)   => {
                ret = self.repository.commit(Some("HEAD"),
                                             &signature,
                                             &signature,
                                             msg,
                                             &tree,
                                             &[]
                );
            }
        }
        ret
    }

    pub fn create_branch(&self, branch_name: &str, src_commit: Oid) {
        let srccommit = self.repository.find_commit(src_commit).unwrap();
        let _ = self.repository.branch(branch_name, &srccommit, true);
    }

    pub fn checkout(&self, branch: &str) {
        let (object, reference) =
            self.repository.revparse_ext(branch).expect("Object not found");

        self.repository.checkout_tree(&object, None).expect("Failed to checkout");

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => self.repository.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => self.repository.set_head_detached(object.id()),
        }
        .expect("Failed to set HEAD");
    }

    pub fn get_last_commit(&self) -> Result<Oid, git2::Error> {
        let h_ref = self.repository.head()?;
        let commit = h_ref.peel_to_commit()?;
        Ok(commit.id())
    }


    pub fn get_workdir(&self) -> &str {
        &self.repository.workdir().unwrap().to_str().unwrap()
    }

}
