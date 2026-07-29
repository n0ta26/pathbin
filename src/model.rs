use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BinaryEntry {
    name: OsString,
    path: PathBuf,
    path_index: usize,
}

impl BinaryEntry {
    pub(crate) fn new(name: OsString, path: PathBuf, path_index: usize) -> Self {
        Self {
            name,
            path,
            path_index,
        }
    }

    pub fn name(&self) -> &OsStr {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn path_index(&self) -> usize {
        self.path_index
    }
}

#[derive(Debug, Default)]
pub struct ScanResult {
    pub(crate) path_entries_total: usize,
    pub(crate) existing_dirs: usize,
    pub(crate) empty_path_entries: usize,
    pub(crate) missing_entries: Vec<PathBuf>,
    pub(crate) non_dir_entries: Vec<PathBuf>,
    pub(crate) unreadable_entries: Vec<PathBuf>,
    pub(crate) broken_symlinks: Vec<PathBuf>,
    pub(crate) binaries: Vec<BinaryEntry>,
    pub(crate) by_name: BTreeMap<OsString, Vec<BinaryEntry>>,
}

impl ScanResult {
    pub fn path_entries_total(&self) -> usize {
        self.path_entries_total
    }

    pub fn existing_dirs(&self) -> usize {
        self.existing_dirs
    }

    pub fn empty_path_entries(&self) -> usize {
        self.empty_path_entries
    }

    pub fn missing_entries(&self) -> &[PathBuf] {
        &self.missing_entries
    }

    pub fn non_dir_entries(&self) -> &[PathBuf] {
        &self.non_dir_entries
    }

    pub fn unreadable_entries(&self) -> &[PathBuf] {
        &self.unreadable_entries
    }

    pub fn broken_symlinks(&self) -> &[PathBuf] {
        &self.broken_symlinks
    }

    pub fn binaries(&self) -> &[BinaryEntry] {
        &self.binaries
    }

    pub fn unique_command_count(&self) -> usize {
        self.by_name.len()
    }

    pub fn duplicate_name_count(&self) -> usize {
        self.by_name
            .values()
            .filter(|entries| entries.len() > 1)
            .count()
    }

    pub fn shadowed_binary_count(&self) -> usize {
        self.by_name
            .values()
            .map(|entries| entries.len().saturating_sub(1))
            .sum()
    }

    pub fn command_matches(&self, command: &OsStr) -> Option<&[BinaryEntry]> {
        self.by_name.get(command).map(Vec::as_slice)
    }

    pub fn duplicate_groups(&self) -> impl Iterator<Item = (&OsStr, &[BinaryEntry])> {
        self.by_name.iter().filter_map(|(name, entries)| {
            if entries.len() > 1 {
                Some((name.as_os_str(), entries.as_slice()))
            } else {
                None
            }
        })
    }

    pub(crate) fn build_name_index(&mut self) {
        self.binaries.sort_by(|left, right| {
            left.path_index()
                .cmp(&right.path_index())
                .then_with(|| left.name().cmp(right.name()))
                .then_with(|| left.path().cmp(right.path()))
        });

        for entry in &self.binaries {
            self.by_name
                .entry(entry.name().to_os_string())
                .or_default()
                .push(entry.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn non_utf8_command_names_remain_distinct_in_the_index() {
        use super::{BinaryEntry, ScanResult};
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        let first = OsStr::from_bytes(b"tool-\xfe");
        let second = OsStr::from_bytes(b"tool-\xff");
        let mut scan = ScanResult::default();
        scan.binaries.push(BinaryEntry::new(
            first.to_os_string(),
            PathBuf::from("first"),
            0,
        ));
        scan.binaries.push(BinaryEntry::new(
            second.to_os_string(),
            PathBuf::from("second"),
            0,
        ));

        scan.build_name_index();

        assert_eq!(scan.unique_command_count(), 2);
        assert_eq!(
            scan.command_matches(first).map(|entries| entries.len()),
            Some(1)
        );
        assert_eq!(
            scan.command_matches(second).map(|entries| entries.len()),
            Some(1)
        );
    }
}
