//! Reading `docs/reference/contracts.json`, and building archives to install.
//!
//! Every catalogued value this crate asserts is **parsed out of the
//! catalogue** rather than retyped into a test. A retyped literal proves the
//! code matches the test; a parsed one proves the code matches the
//! specification, and it fails loudly when a contract is renamed or removed.
//!
//! The second half of this module builds `.tar.bz2` fixtures in memory. A
//! checked-in archive would be a binary blob a reviewer cannot read, and the
//! interesting cases — a member named `../../escape`, a symlink wearing a
//! member's name, a truncated stream — are ones no real archive contains.

#![allow(dead_code)]

use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use bzip2::Compression;
use bzip2::write::BzEncoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tar::{EntryType, Header};
use via_wake_word::{ModelArtifact, ModelFiles, WAKE_WORD_MODEL_FILES};

// ── the catalogue ───────────────────────────────────────────────────────────

/// One catalogued contract.
#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    /// `file-path`, `http-route`, `default-value`, `env-var`, …
    pub kind: String,
    /// The contract's name.
    pub name: String,
    /// The value, or a description of it.
    #[serde(rename = "exactValue")]
    pub exact_value: String,
    /// Where upstream defines it.
    pub file: String,
    /// Why it is external.
    pub why: String,
}

fn catalogue() -> &'static Vec<Contract> {
    static CATALOGUE: OnceLock<Vec<Contract>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("reference")
            .join("contracts.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
        serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} must be a contract array: {error}", path.display()))
    })
}

/// The one contract with this name **and** kind.
///
/// Panics when it is missing or ambiguous, because either means the catalogue
/// moved under a test that claims to assert it.
#[must_use]
pub fn contract_of_kind(kind: &str, name: &str) -> &'static Contract {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|contract| contract.name == name && contract.kind == kind)
        .collect();
    match matches.as_slice() {
        [only] => only,
        [] => panic!("no `{kind}` contract named `{name}` in docs/reference/contracts.json"),
        many => panic!("{} `{kind}` contracts named `{name}`", many.len()),
    }
}

impl Contract {
    /// The `'…'`-quoted fragment that follows `marker`.
    #[must_use]
    pub fn quoted_after(&self, marker: &str) -> String {
        let rest = self.after(marker);
        let mut chars = rest.chars();
        assert_eq!(
            chars.next(),
            Some('\''),
            "`{marker}` in contract `{}` is not followed by a quoted value:\n{}",
            self.name,
            self.exact_value
        );
        let mut value = String::new();
        for character in chars {
            if character == '\'' {
                return value;
            }
            value.push(character);
        }
        panic!(
            "the value after `{marker}` in contract `{}` is unterminated",
            self.name
        )
    }

    /// The integer that follows `marker`.
    #[must_use]
    pub fn number_after(&self, marker: &str) -> u64 {
        let digits: String = self
            .after(marker)
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no integer after `{marker}` in contract `{}`", self.name))
    }

    /// The decimal that follows `marker`.
    #[must_use]
    pub fn decimal_after(&self, marker: &str) -> f64 {
        let digits: String = self
            .after(marker)
            .chars()
            .take_while(|character| character.is_ascii_digit() || *character == '.')
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no decimal after `{marker}` in contract `{}`", self.name))
    }

    /// The whole value with the catalogue's `\n` escapes turned into real
    /// newlines.
    #[must_use]
    pub fn unescaped(&self) -> String {
        self.exact_value.replace("\\n", "\n")
    }

    /// The `'…'`-quoted fragments of the `why` field, in order.
    ///
    /// Several catalogue rows put the user-facing message in the rationale
    /// rather than the value — `WAKE_WORD_MODEL_SHA256`'s *"mismatch throws
    /// '唤醒词模型校验失败'"*, for one — and that message is still the
    /// contract.
    #[must_use]
    pub fn why_quoted(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = self.why.as_str();
        while let Some(start) = rest.find('\'') {
            rest = &rest[start + 1..];
            let Some(end) = rest.find('\'') else { break };
            out.push(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
        out
    }

    /// Assert the contract's value mentions `needle`.
    pub fn assert_mentions(&self, needle: &str) {
        assert!(
            self.exact_value.contains(needle),
            "contract `{}` no longer mentions `{needle}`:\n{}",
            self.name,
            self.exact_value
        );
    }

    /// Assert the contract's rationale mentions `needle`.
    pub fn assert_why_mentions(&self, needle: &str) {
        assert!(
            self.why.contains(needle),
            "contract `{}` rationale no longer mentions `{needle}`:\n{}",
            self.name,
            self.why
        );
    }

    fn after(&self, marker: &str) -> &str {
        let start = self
            .exact_value
            .find(marker)
            .unwrap_or_else(|| panic!("`{marker}` is not in contract `{}`", self.name))
            + marker.len();
        &self.exact_value[start..]
    }
}

// ── archive fixtures ────────────────────────────────────────────────────────

/// One member of a fixture archive.
pub struct Member {
    /// The name as it appears in the tar header, `..` and all.
    pub name: String,
    /// The bytes, for a regular file.
    pub body: Vec<u8>,
    /// What kind of entry it is.
    pub kind: EntryType,
    /// The link target, for a symlink or hard link.
    pub link: Option<String>,
}

impl Member {
    /// A regular file.
    #[must_use]
    pub fn file(name: &str, body: &[u8]) -> Self {
        Self {
            name: name.to_owned(),
            body: body.to_vec(),
            kind: EntryType::Regular,
            link: None,
        }
    }

    /// A directory.
    #[must_use]
    pub fn directory(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            body: Vec::new(),
            kind: EntryType::Directory,
            link: None,
        }
    }

    /// A symlink pointing somewhere.
    #[must_use]
    pub fn symlink(name: &str, target: &str) -> Self {
        Self {
            name: name.to_owned(),
            body: Vec::new(),
            kind: EntryType::Symlink,
            link: Some(target.to_owned()),
        }
    }
}

/// Build a `.tar.bz2` in memory from `members`.
///
/// The member name is written **straight into the header's name field** rather
/// than through `Builder::append_data`, because `tar-rs`'s writer refuses a
/// path containing `..` — reasonably, since it is a writer. A hostile archive
/// is exactly what the extractor has to survive, so the fixture builder has to
/// be able to produce one.
#[must_use]
pub fn tar_bz2(members: &[Member]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for member in members {
        let name = member.name.as_bytes();
        assert!(
            name.len() < 100,
            "fixture member names stay inside the 100-byte tar name field"
        );

        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_entry_type(member.kind);
        if let Some(target) = &member.link {
            header
                .set_link_name(target)
                .expect("the fixture link target is representable");
        }
        let body: &[u8] = if member.kind == EntryType::Regular {
            &member.body
        } else {
            &[]
        };
        header.set_size(body.len() as u64);
        header
            .as_gnu_mut()
            .expect("Header::new_gnu is a GNU header")
            .name[..name.len()]
            .copy_from_slice(name);
        header.set_cksum();

        builder.append(&header, body).expect("append the member");
    }
    let tar = builder.into_inner().expect("finish the tar");

    let mut encoder = BzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&tar).expect("compress the tar");
    encoder.finish().expect("finish the bzip2 stream")
}

/// The four archive members of `files`, each carrying recognisable bytes.
#[must_use]
pub fn complete_members(files: &ModelFiles) -> Vec<Member> {
    files
        .archived()
        .iter()
        .map(|name| Member::file(name, body_for(name).as_bytes()))
        .collect()
}

/// The bytes a fixture member carries.
///
/// Three of the four carry their own name, so a test can tell which file landed
/// where. `tokens.txt` carries a **real token inventory** — see
/// [`fixture_tokens_file`] — because the installer validates the keyword file
/// against it before writing anything, and a fixture that could not be
/// validated would only prove the guard fires.
#[must_use]
pub fn body_for(name: &str) -> String {
    if name == WAKE_WORD_MODEL_FILES.tokens.as_ref() {
        return fixture_tokens_file();
    }
    format!("fixture bytes for {name}\n")
}

/// A `tokens.txt` in the real format: `<symbol> <id>`, one per line.
///
/// The three control symbols the catalogued model opens with, then the ASCII
/// lowercase letters — enough for every fixture token line in this crate's
/// tests, and shaped exactly like the file the model ships.
#[must_use]
pub fn fixture_tokens_file() -> String {
    let mut out = String::from("<blk> 0\n<sos/eos> 1\n<unk> 2\n");
    for (index, letter) in ('a'..='z').enumerate() {
        out.push_str(&format!("{letter} {}\n", index + 3));
    }
    out
}

/// The SHA-256 of `bytes`, lowercase hex.
#[must_use]
pub fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// An artifact with upstream's file names, a fixture id, an unroutable release
/// host, and the digest of `archive`.
///
/// The host is `.invalid` (RFC 2606) on purpose: if a test ever escaped its
/// [`ScriptedFetch`](via_wake_word::ScriptedFetch), the request would fail to
/// resolve rather than reach the real release page.
#[must_use]
pub fn fixture_artifact(archive: &[u8]) -> ModelArtifact {
    ModelArtifact {
        id: Cow::Borrowed("fixture-kws-model-2026-01-01"),
        release_base: Cow::Borrowed("https://releases.invalid/kws-models"),
        archive_suffix: Cow::Borrowed(".tar.bz2"),
        sha256: Cow::Owned(digest(archive)),
        files: WAKE_WORD_MODEL_FILES,
    }
}

/// A second artifact, so "two locales, two models" is testable.
#[must_use]
pub fn other_artifact(archive: &[u8]) -> ModelArtifact {
    ModelArtifact {
        id: Cow::Borrowed("fixture-kws-model-other"),
        sha256: Cow::Owned(digest(archive)),
        ..fixture_artifact(archive)
    }
}
