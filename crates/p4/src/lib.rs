//! The Perforce plugin: the Source control pane, the Explorer's marks, diffs and the `vcs.*`
//! convention (Schemify's docs/plugins/vcs.md) for a Perforce working copy, all the shared
//! driver's over the `P4` row of `schemify_vcs::backends`.

#![allow(
    missing_docs,
    reason = "guest::export! writes the ABI's undocumented exports"
)]

#[cfg(target_arch = "wasm32")]
schemify_abi::guest::export!(on_event);

/// The entry point: every event is the driver's.
#[cfg(target_arch = "wasm32")]
fn on_event(
    ev: schemify_abi::Event<'_>,
    host: &schemify_abi::guest::Host,
    out: &mut schemify_abi::Out,
) {
    let _ = schemify_vcs::driver::on_event(&schemify_vcs::backends::P4, "", ev, host, out);
}
