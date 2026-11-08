//! The plugin's events (wasm only): `git.setup` here, everything else the shared driver's
//! over the [`GIT`] row (`schemify_vcs::driver`).
//!
//! Contents: `fn on_event` (via `export!`); private `Step`.

use std::cell::Cell;

use schemify_abi::guest::{self, Host};
use schemify_abi::{Event, Level, Out};
use schemify_vcs::backends::GIT;
use schemify_vcs::driver;

use crate::with_merge_lines;

guest::export!(on_event);

/// The command this plugin adds to the driver's.
const SETUP: &str = "command git.setup \"Set up the semantic merge driver in this clone\"\n";

/// The merge driver `git.setup` configures: git runs it for the design files.
const DRIVER: [(&str, &str); 2] = [
    ("merge.schemify.name", "Schemify semantic merge"),
    ("merge.schemify.driver", "schemify merge-driver %O %A %B %P"),
];

/// A `git.setup` step.
#[derive(Clone, Copy)]
enum Step {
    /// `git config` of [`DRIVER`] row `i`.
    Config(usize),
    /// The read of `.gitattributes`.
    Attrs,
}

thread_local! {
    /// `git.setup`'s step in flight and its handle. The guest is single-threaded.
    static SETUP_STEP: Cell<Option<(u32, Step)>> = const { Cell::new(None) };
}

// ---- private ----

/// The entry point `export!` names: `git.setup` and its results here, the rest the driver's.
fn on_event(ev: Event<'_>, host: &Host, out: &mut Out) {
    match ev {
        Event::Command {
            name: "git.setup", ..
        } => start_setup(host, out),
        Event::IoDone {
            handle,
            status,
            bytes,
        } if SETUP_STEP.get().is_some_and(|(h, _)| h == handle) => {
            let step = SETUP_STEP.take().map(|(_, s)| s).expect("checked above");
            setup_step(host, out, step, status, bytes);
        }
        _ => {
            let _ = driver::on_event(&GIT, SETUP, ev, host, out);
        }
    }
}

/// `git.setup`: the clone's merge driver (git never reads driver commands from the
/// repository, for security), then the `.gitattributes` lines it lacks, one step after the
/// other ([`setup_step`] takes each result).
fn start_setup(host: &Host, out: &mut Out) {
    if SETUP_STEP.get().is_some() {
        out.log_at(Level::Warn, "git.setup: already running");
        return;
    }
    let (key, value) = DRIVER[0];
    match host.spawn(&["git", "config", key, value], b"") {
        Ok(h) => SETUP_STEP.set(Some((h, Step::Config(0)))),
        Err(_) => out.log_at(Level::Error, "git.setup: needs the spawn:git grant"),
    }
}

/// Takes one `git.setup` result and starts the next step: the next `git config`, the read of
/// `.gitattributes` (missing: empty), then the write of the lines it lacks.
fn setup_step(host: &Host, out: &mut Out, step: Step, status: i32, bytes: &[u8]) {
    assert!(SETUP_STEP.get().is_none(), "the step was taken off");
    match step {
        Step::Config(_) if status != 0 => {
            out.log_at(Level::Error, "git config failed (is this a git clone?)");
        }
        Step::Config(i) if i + 1 < DRIVER.len() => {
            let (key, value) = DRIVER[i + 1];
            let next = host.spawn(&["git", "config", key, value], b"");
            SETUP_STEP.set(next.ok().map(|h| (h, Step::Config(i + 1))));
        }
        Step::Config(_) => {
            SETUP_STEP.set(
                host.fs_read(".gitattributes")
                    .ok()
                    .map(|h| (h, Step::Attrs)),
            );
        }
        Step::Attrs => {
            let text = if status < 0 {
                String::new()
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            };
            let wrote = with_merge_lines(&text);
            if wrote.len() != text.len()
                && host.fs_write(".gitattributes", wrote.as_bytes()).is_err()
            {
                out.log_at(Level::Error, "git.setup: needs the write_project grant");
                return;
            }
            out.log("merge driver configured; commit .gitattributes so clones share it");
        }
    }
}
