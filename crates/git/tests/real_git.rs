//! The `GIT` row against real git: a temporary repository, the row's own argv (filled as the
//! plugin fills them), and its parsers over what git printed. The oracle is the repository the
//! test made: the files it changed, the commits it wrote, the lines each commit added.
//! Skipped where git is missing.

use std::path::{Path, PathBuf};
use std::process::Command;

use schemify_vcs::backends::GIT;
use schemify_vcs::{annotate, fill};

/// A fresh directory under the system temp directory.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("schemify-git-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("writable temp dir");
    dir
}

/// Runs `argv` in `dir` with a fixed identity and clock; stdout when it exits 0.
fn run(dir: &Path, argv: &[String]) -> Option<Vec<u8>> {
    let out = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Ann Lee")
        .env("GIT_AUTHOR_EMAIL", "ann@example.com")
        .env("GIT_COMMITTER_NAME", "Ann Lee")
        .env("GIT_COMMITTER_EMAIL", "ann@example.com")
        .env("GIT_AUTHOR_DATE", "1700000000 +0000")
        .env("GIT_COMMITTER_DATE", "1700000000 +0000")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .ok()?;
    out.status.success().then_some(out.stdout)
}

fn git(dir: &Path, args: &[&str]) -> bool {
    let argv: Vec<String> = std::iter::once("git")
        .chain(args.iter().copied())
        .map(String::from)
        .collect();
    run(dir, &argv).is_some()
}

/// A repository whose project lives in `proj/`: `a.lua` committed twice, then modified;
/// `b.lua` committed then deleted; `c.lua` untracked; `outside.txt` changed outside the
/// project. Returns the project directory, or `None` without git.
fn repo(tag: &str) -> Option<PathBuf> {
    let top = scratch(tag);
    let proj = top.join("proj");
    std::fs::create_dir_all(&proj).expect("writable");
    if !git(&top, &["init", "-q"]) {
        return None;
    }
    std::fs::write(proj.join("a.lua"), "one\n").expect("writable");
    std::fs::write(proj.join("b.lua"), "gone\n").expect("writable");
    std::fs::write(top.join("outside.txt"), "x\n").expect("writable");
    assert!(git(&top, &["add", "."]) && git(&top, &["commit", "-qm", "first"]));
    std::fs::write(proj.join("a.lua"), "one\ntwo\n").expect("writable");
    assert!(git(&top, &["commit", "-qam", "second: add two"]));
    std::fs::write(proj.join("a.lua"), "one\ntwo\nthree\n").expect("writable");
    std::fs::remove_file(proj.join("b.lua")).expect("removable");
    std::fs::write(proj.join("c.lua"), "new\n").expect("writable");
    std::fs::write(top.join("outside.txt"), "y\n").expect("writable");
    Some(proj)
}

#[test]
fn detect_and_status_list_the_project_changes_only() {
    let Some(proj) = repo("status") else { return };
    let prefix = (GIT.parse_root)(&run(&proj, &fill(GIT.root, "", "", "")).expect("a repository"))
        .expect("a prefix");
    assert_eq!(prefix, "proj/");
    let out = run(&proj, &fill(GIT.status, "", "", "")).expect("status runs");
    let mut rows = (GIT.parse_status)(&out, &prefix);
    rows.sort();
    let want = [('D', "b.lua"), ('M', "a.lua"), ('U', "c.lua")];
    assert_eq!(
        rows,
        want.map(|(m, p)| (m, p.to_owned())),
        "outside.txt is not the project's"
    );
    let outside = scratch("none");
    assert!(
        run(&outside, &fill(GIT.root, "", "", "")).is_none(),
        "not a repository"
    );
}

#[test]
fn base_diff_and_show_print_the_revisions() {
    let Some(proj) = repo("show") else { return };
    let prefix = "proj/";
    let base = run(&proj, &fill(GIT.base, "a.lua", prefix, "")).expect("HEAD has a.lua");
    assert_eq!(base, b"one\ntwo\n");
    let first = run(&proj, &fill(GIT.show, "a.lua", prefix, "HEAD~1")).expect("HEAD~1 has a.lua");
    assert_eq!(first, b"one\n");
    let diff = run(&proj, &fill(GIT.diff, "a.lua", prefix, "")).expect("diff runs");
    assert!(String::from_utf8_lossy(&diff).contains("+three"));
}

/// Annotate joined with the log: each line's commit, author, time and summary, as the
/// annotate file carries them; the uncommitted line has none.
#[test]
fn annotate_joins_the_log_per_line() {
    let Some(proj) = repo("blame") else { return };
    let log = (GIT.parse_log)(&run(&proj, &fill(GIT.log, "a.lua", "proj/", "")).expect("log"));
    let summaries: Vec<&str> = log.iter().map(|r| r.summary.as_str()).collect();
    assert_eq!(summaries, ["second: add two", "first"], "newest first");
    assert!(
        log.iter()
            .all(|r| r.author == "Ann Lee" && r.time == 1_700_000_000)
    );
    let revs = (GIT.parse_annotate)(
        &run(&proj, &fill(GIT.annotate, "a.lua", "proj/", "")).expect("blame"),
    );
    assert_eq!(revs.len(), 3, "one revision per line of the working file");
    let rows = annotate::join(&revs, &log);
    let got: Vec<&str> = rows.iter().map(|r| r.summary.as_str()).collect();
    assert_eq!(got, ["first", "second: add two", "not committed"]);
    let work = std::fs::read(proj.join("a.lua")).expect("readable");
    let file = annotate::text(&work, "git", &rows);
    assert_eq!(annotate::parse(&file, &work).expect("fresh"), rows);
}
