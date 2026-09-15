//! Interpolation.
//!
//! # Syntax
//!
//! One form: `{name}`, where `name` is `[a-z0-9_]+`. A literal brace is
//! doubled — `{{` renders `{`, `}}` renders `}`. There is no formatting spec,
//! no nesting and no positional form; every argument is a `&str` the caller
//! has already rendered.
//!
//! Upstream interpolates with JavaScript's `${…}`, which is a template
//! *expression* and cannot be reproduced by a data-driven catalog. `{name}`
//! was chosen over `${name}` and `{{name}}` because it is the spelling Rust
//! programmers already read in `format!`, and over `%s`/`%1$s` because a named
//! hole survives a translator reordering the sentence — which Korean and
//! Chinese word order both force.
//!
//! # Failure is loud
//!
//! Every mismatch between the template and the arguments is an error:
//!
//! - a `{name}` with no argument ([`FormatError::MissingArgument`]);
//! - an argument the template never uses ([`FormatError::UnusedArgument`]);
//! - the same argument name twice ([`FormatError::DuplicateArgument`]);
//! - a malformed template (unterminated, empty or non-snake-case placeholder,
//!   or a stray `}`), which `build.rs` already rejects for catalog values but
//!   which [`render`] must still answer for a template from anywhere else.
//!
//! A message that silently drops a value is worse than one that refuses to
//! render: the first ships a sentence with a hole in it to a user, the second
//! shows up in a test.

use core::fmt;

/// Why a template could not be rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The template contains `{name}` and no argument was supplied for it.
    MissingArgument {
        /// The placeholder name.
        name: String,
    },
    /// An argument was supplied that the template never mentions. Usually a
    /// renamed placeholder that only got renamed on one side.
    UnusedArgument {
        /// The argument name.
        name: String,
    },
    /// The same argument name was supplied twice. Which one wins would be
    /// arbitrary, so neither does.
    DuplicateArgument {
        /// The argument name.
        name: String,
    },
    /// A `{` with no matching `}`.
    UnterminatedPlaceholder {
        /// Byte offset of the `{`.
        at: usize,
    },
    /// `{}` — a placeholder with no name.
    EmptyPlaceholder {
        /// Byte offset of the `{`.
        at: usize,
    },
    /// A placeholder name outside `[a-z0-9_]+`.
    InvalidPlaceholderName {
        /// The offending name, as written.
        name: String,
        /// Byte offset of the `{`.
        at: usize,
    },
    /// A `}` that closes nothing. Write `}}` for a literal one.
    StrayCloseBrace {
        /// Byte offset of the `}`.
        at: usize,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::MissingArgument { name } => {
                write!(f, "no value for placeholder `{{{name}}}`")
            }
            FormatError::UnusedArgument { name } => {
                write!(f, "argument `{name}` is not used by this message")
            }
            FormatError::DuplicateArgument { name } => {
                write!(f, "argument `{name}` was supplied more than once")
            }
            FormatError::UnterminatedPlaceholder { at } => {
                write!(f, "unterminated `{{` at byte {at}")
            }
            FormatError::EmptyPlaceholder { at } => {
                write!(f, "empty placeholder `{{}}` at byte {at}")
            }
            FormatError::InvalidPlaceholderName { name, at } => {
                write!(
                    f,
                    "placeholder `{{{name}}}` at byte {at} is not a [a-z0-9_]+ name"
                )
            }
            FormatError::StrayCloseBrace { at } => {
                write!(
                    f,
                    "stray `}}` at byte {at} — write `}}}}` for a literal brace"
                )
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// Interpolate `args` into `template`.
///
/// This is the engine [`try_format`](crate::try_format) runs on catalog text.
/// It is public because a caller with a template from somewhere else — a
/// per-backend onboarding hint, a value read from configuration — should use
/// the same grammar and get the same errors rather than inventing a second
/// one.
///
/// ```
/// use via_i18n::render;
///
/// assert_eq!(render("a {b} c", &[("b", "B")]).as_deref(), Ok("a B c"));
/// assert_eq!(render("{{literal}}", &[]).as_deref(), Ok("{literal}"));
/// assert!(render("a {b} c", &[]).is_err());
/// assert!(render("a c", &[("b", "B")]).is_err());
/// ```
pub fn render(template: &str, args: &[(&str, &str)]) -> Result<String, FormatError> {
    for (index, (name, _)) in args.iter().enumerate() {
        if args[..index].iter().any(|(earlier, _)| earlier == name) {
            return Err(FormatError::DuplicateArgument {
                name: (*name).to_owned(),
            });
        }
    }

    let mut used = vec![false; args.len()];
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => {
                out.push('{');
                index += 2;
            }
            b'}' if bytes.get(index + 1) == Some(&b'}') => {
                out.push('}');
                index += 2;
            }
            b'}' => return Err(FormatError::StrayCloseBrace { at: index }),
            b'{' => {
                let rest = &template[index + 1..];
                let Some(len) = rest.find('}') else {
                    return Err(FormatError::UnterminatedPlaceholder { at: index });
                };
                let name = &rest[..len];
                if name.is_empty() {
                    return Err(FormatError::EmptyPlaceholder { at: index });
                }
                if !name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    return Err(FormatError::InvalidPlaceholderName {
                        name: name.to_owned(),
                        at: index,
                    });
                }
                let Some(position) = args.iter().position(|(arg, _)| *arg == name) else {
                    return Err(FormatError::MissingArgument {
                        name: name.to_owned(),
                    });
                };
                used[position] = true;
                out.push_str(args[position].1);
                index += 1 + len + 1;
            }
            _ => {
                // Copy the whole run of ordinary bytes, braces excluded. UTF-8
                // continuation bytes are never `{` or `}`, so slicing on the
                // next brace is always a char boundary.
                let start = index;
                while index < bytes.len() && bytes[index] != b'{' && bytes[index] != b'}' {
                    index += 1;
                }
                out.push_str(&template[start..index]);
            }
        }
    }

    if let Some(position) = used.iter().position(|seen| !seen) {
        return Err(FormatError::UnusedArgument {
            name: args[position].0.to_owned(),
        });
    }

    Ok(out)
}
