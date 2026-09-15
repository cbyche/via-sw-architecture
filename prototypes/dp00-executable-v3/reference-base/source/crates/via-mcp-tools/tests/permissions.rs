//! The permission broker's auto-approve carve-out.
//!
//! `docs/architecture.md` §13 names this the highest-risk item in the crate:
//!
//! > the five MCP tool names (the permission broker auto-approves them via
//! > *three* name-shape matches — exact, `endsWith("__<name>")`,
//! > `startsWith("<name> (")` — so a partial rename wedges the coordinator)
//!
//! "Wedges" is literal. A coordination call that stops being auto-approved
//! becomes a `session/request_permission` sent to a voice frontend that is not
//! waiting for one, and the delegation hangs until the request times out. So
//! this file asserts all three shapes for all five names, from both ends: no
//! served tool may fail to match, and no plausible foreign name may match.
//!
//! The broker itself is not here — it belongs to whoever owns
//! `session/request_permission`. What is here is the **only** implementation of
//! the question it asks, so that the answer cannot be given twice and
//! differently.

use via_mcp_tools::{
    SESSION_TOOL_NAMES, SESSION_TOOL_SERVER, SessionTool, SessionToolMatch, is_session_tool,
    match_session_tool, tool_definitions,
};

/// Every tool, presented in every shape a backend is known to use.
#[test]
fn all_three_shapes_match_all_five_names() {
    for tool in SessionTool::ALL {
        let name = tool.name();
        for (presented, shape) in [
            (name.to_owned(), SessionToolMatch::Exact),
            (
                format!("mcp__{SESSION_TOOL_SERVER}__{name}"),
                SessionToolMatch::NamespacePrefixed,
            ),
            (
                format!("{name} (build the thing)"),
                SessionToolMatch::TitlePrefixed,
            ),
        ] {
            assert_eq!(
                match_session_tool(&presented),
                Some((tool, shape)),
                "{presented} did not match {name} as {shape:?}",
            );
            assert!(is_session_tool(&presented), "{presented}");
        }
    }
}

/// The namespace shape is `endsWith`, so any namespace works — not only VIA's.
#[test]
fn the_namespace_shape_does_not_assume_which_namespace() {
    for prefix in [
        "mcp__via__",
        "mcp__anything__",
        "x__",
        "__",
        "some.tool__",
        "mcp__a__b__",
    ] {
        let presented = format!("{prefix}via_session_status");
        assert_eq!(
            match_session_tool(&presented),
            Some((
                SessionTool::SessionStatus,
                SessionToolMatch::NamespacePrefixed,
            )),
            "{presented}",
        );
    }
}

/// The title shape needs the space-and-paren, not merely a prefix.
#[test]
fn the_title_shape_needs_the_space_and_paren() {
    assert_eq!(
        match_session_tool("via_session_cancel ()"),
        Some((SessionTool::SessionCancel, SessionToolMatch::TitlePrefixed)),
    );
    assert_eq!(
        match_session_tool("via_session_cancel (delegation_id=acp_run_1)"),
        Some((SessionTool::SessionCancel, SessionToolMatch::TitlePrefixed)),
    );
    // A prefix without the paren is a different tool's name, not ours.
    assert!(!is_session_tool("via_session_cancel_all"));
    assert!(!is_session_tool("via_session_cancel-x"));
    assert!(!is_session_tool("via_session_cancel x)"));
    // The bare name still matches, but as `Exact` rather than as a title.
    assert_eq!(
        match_session_tool("via_session_cancel").map(|(_, shape)| shape),
        Some(SessionToolMatch::Exact),
    );
}

#[test]
fn a_bare_name_that_is_not_ours_is_never_auto_approved() {
    for name in [
        "",
        "   ",
        "bash",
        "read",
        "glob",
        "grep",
        "write",
        "mcp__via__bash",
        "via",
        "via_",
        "via_session",
        "via_sessions",
        "via_sessions_listing",
        "via_session_starter",
        "session_start",
        "VIA_SESSION_START",
        "via_session_start_",
        "_via_session_start",
        "via_session_start__",
        "(via_session_start)",
        "open-computer-use__screenshot",
    ] {
        assert!(!is_session_tool(name), "{name:?} was auto-approved");
    }
}

/// Upstream trims the presented name once before testing it
/// (`permission-broker.mjs:38`), and so must VIA — including for the
/// whitespace ECMAScript calls whitespace and Rust does not.
#[test]
fn the_presented_name_is_cleaned_first() {
    for presented in [
        " via_session_send ",
        "\tvia_session_send\n",
        "\u{feff}via_session_send",
        "via_session_send\u{feff}",
        "\u{2028}via_session_send\u{2029}",
    ] {
        assert!(is_session_tool(presented), "{presented:?}");
    }
    // U+0085 is Rust's `char::is_whitespace` but not ECMAScript's `\s`, so a
    // name padded with it does **not** trim and does **not** match — matching
    // upstream, and matching `via-downstream`'s `clean`.
    assert!(!is_session_tool("\u{85}via_session_send"));
}

/// Both directions of the one-array invariant, over the shipped surface.
#[test]
fn the_served_surface_and_the_allow_list_are_the_same_five() {
    let served: Vec<String> = tool_definitions()
        .iter()
        .filter_map(|definition| definition["name"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(served, SESSION_TOOL_NAMES.to_vec());
    for name in &served {
        assert!(is_session_tool(name), "{name} is served but not approved");
    }
    for name in SESSION_TOOL_NAMES {
        assert!(
            served.iter().any(|entry| entry == name),
            "{name} is approved but not served",
        );
    }
}

/// No two tool names may be extensions of each other in a way that makes the
/// three shapes ambiguous. If one ever were, a rename could silently route a
/// permission decision to the wrong tool.
#[test]
fn no_name_shadows_another_under_any_shape() {
    for tool in SessionTool::ALL {
        for other in SessionTool::ALL {
            if tool == other {
                continue;
            }
            assert!(
                !other.name().ends_with(&format!("__{}", tool.name())),
                "{} shadows {}",
                other.name(),
                tool.name(),
            );
            assert!(
                !other.name().starts_with(&format!("{} (", tool.name())),
                "{} shadows {}",
                other.name(),
                tool.name(),
            );
        }
    }
}

/// The match reports *which* tool, not merely that it was one — a broker that
/// wants to log or meter by tool needs that, and getting it from a second
/// lookup would be a second source of truth.
#[test]
fn a_match_names_the_tool_it_matched() {
    assert_eq!(
        match_session_tool("mcp__via__via_sessions_list").map(|(tool, _)| tool),
        Some(SessionTool::SessionsList),
    );
    assert_eq!(
        match_session_tool("via_session_cancel (x)").map(|(tool, _)| tool),
        Some(SessionTool::SessionCancel),
    );
    assert_eq!(match_session_tool("bash"), None);
}

#[test]
fn the_shape_names_are_stable() {
    assert_eq!(SessionToolMatch::Exact.as_str(), "exact");
    assert_eq!(
        SessionToolMatch::NamespacePrefixed.as_str(),
        "namespace_prefixed",
    );
    assert_eq!(SessionToolMatch::TitlePrefixed.as_str(), "title_prefixed");
    assert_eq!(SessionToolMatch::ALL.len(), 3);
}
