//! The git plugin: everything git in Schemify, on the generic plugin hooks. Schemify has no
//! version control of its own; it embeds this module as a pinned built-in that is on by
//! default (granted `spawn:git`, `read_project`, `write_project` and `commands`).
//!
//! The pane, the Explorer's marks, the diffs and the `vcs.*` convention are the shared driver's
//! over the `GIT` row (`schemify_vcs`); this crate adds `git.setup`, the semantic merge driver.
//!
//! Contents: `mod plugin` (events; wasm only), `fn with_merge_lines`.

#![allow(
    missing_docs,
    reason = "guest::export! writes the ABI's undocumented exports"
)]
#![deny(unsafe_code)]
// not `forbid`: the SDK's `export!` allows `unsafe(no_mangle)`
// Natively only the tests build (the events are wasm only), so the pure parts look unused.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

#[cfg(target_arch = "wasm32")]
mod plugin;

/// `text` (a `.gitattributes`) with a `*.<ext> merge=schemify` line for each design file kind
/// it lacks; lines are only added.
pub(crate) fn with_merge_lines(text: &str) -> String {
    let mut text = text.to_owned();
    let before = text.len();
    for ext in ["shprim", "shcomp", "shbench"] {
        let line = format!("*.{ext} merge=schemify");
        if text.lines().any(|l| l.trim() == line) {
            continue;
        }
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
    }
    assert!(text.len() >= before, "lines are only added");
    text
}

#[cfg(test)]
mod tests {
    /// `git.setup` only adds the lines a `.gitattributes` lacks, after a missing final newline.
    #[test]
    fn merge_lines_are_added_once() {
        let all = "*.shprim merge=schemify\n*.shcomp merge=schemify\n*.shbench merge=schemify\n";
        assert_eq!(super::with_merge_lines(""), all);
        assert_eq!(super::with_merge_lines(all), all);
        assert_eq!(
            super::with_merge_lines("*.png binary\n*.shcomp merge=schemify"),
            "*.png binary\n*.shcomp merge=schemify\n*.shprim merge=schemify\n*.shbench merge=schemify\n"
        );
    }
}
