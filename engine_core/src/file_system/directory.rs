use std::path::PathBuf;
use std::{env, slice};

const MAX_DIRS: usize = 128;

#[derive(Debug)]
pub struct IwadDirs {
    dirs: Vec<PathBuf>,
}

impl IwadDirs {
    pub(crate) fn new() -> Self {
        Self {
            dirs: Vec::with_capacity(MAX_DIRS),
        }
    }

    fn add_path_buf(&mut self, path: PathBuf) {
        if self.dirs.len() < MAX_DIRS {
            self.dirs.push(path);
        }
    }

    pub(crate) fn add_dir<T: Into<PathBuf>>(&mut self, dir: T) {
        self.add_path_buf(dir.into());
    }

    pub(crate) fn add_path(&mut self, path: &str, suffix: &str) {
        for mut path in env::split_paths(path) {
            path.push(suffix);
            self.add_path_buf(path);
        }
    }
}

impl<'a> IntoIterator for &'a IwadDirs {
    type Item = &'a PathBuf;
    type IntoIter = slice::Iter<'a, PathBuf>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.dirs.iter()
    }
}
