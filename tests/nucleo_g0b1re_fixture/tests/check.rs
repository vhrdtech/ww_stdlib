//! The `nucleo_g0b1re_api` fixture against its old-format snapshots (EVO-13, EVO-70): nucleo_g0b1re_template's API
//! 0.4.0 as deployed firmware runs it, with the snapshots saved by a wire_weaver from before
//! `ItemEnum::future_version` (EVO-10). It follows this repository's crates by path on purpose, so a stdlib change
//! reaches it at once; its pinned signatures are refreshed by `just refresh-fixture-pins`, never by hand.

use std::path::{Path, PathBuf};
use wire_weaver_evolution::snapshot_files::{self, SNAPSHOTS_DIR};
use wire_weaver_evolution::{Change, Check, CrateCache, Outcome, Report, pins};
use ww_snapshots::ww_self::ApiLevelLocationOwned;

/// A crate in a fresh directory, removed on drop.
struct TestCrate {
    dir: PathBuf,
}

impl TestCrate {
    fn new(name: &str) -> Self {
        // under the workspace, so the walker finds its Cargo.lock and [patch]es for git dependencies
        let dir = repo()
            .join("target/ww_evolution_check_tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        TestCrate { dir }
    }

    fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for TestCrate {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The repository root.
fn repo() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn check_current(path: &Path) -> (Outcome, Result<String, String>) {
    let outcome = Check::new(path).require_current(true).run().unwrap();
    let verdict = outcome.verdict();
    (outcome, verdict)
}

fn has_all(text: &str, expected: &[&str]) {
    for expected in expected {
        assert!(text.contains(expected), "'{expected}' not in:\n{text}");
    }
}

fn unrecorded(outcome: &Outcome) -> Vec<String> {
    let Outcome::Compared(comparison) = outcome else {
        panic!("{outcome:?}")
    };
    comparison
        .report
        .unrecorded
        .iter()
        .map(|u| u.to_string())
        .collect()
}

/// nucleo_g0b1re_template's API 0.4.0 with its snapshots, saved by a wire_weaver from before
/// `ItemEnum::future_version` (EVO-10). It follows the live stdlib crates by path on purpose, so a stdlib change
/// reaches it at once; its pinned stdlib signatures are refreshed by `just refresh-fixture-pins` (EVO-70), never by
/// hand.
fn old_format_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/nucleo_g0b1re_api")
}

fn old_format_snapshot(crate_dir: &Path) -> PathBuf {
    crate_dir
        .join(SNAPSHOTS_DIR)
        .join("nucleo_g0b1re_api_0_4_0.ron")
}

/// The fixture copied into `c`, its path dependencies made absolute, with `lib_rs` changed by `edit`.
fn copy_fixture(c: &TestCrate, edit: impl Fn(String) -> String) {
    copy_fixture_with(c, edit, |cargo_toml| cargo_toml);
}

/// [copy_fixture] with its `Cargo.toml` (dependencies already absolute) changed by `edit_toml` too.
fn copy_fixture_with(
    c: &TestCrate,
    edit: impl Fn(String) -> String,
    edit_toml: impl Fn(String) -> String,
) {
    let fixture = old_format_fixture();
    let cargo_toml = std::fs::read_to_string(fixture.join("Cargo.toml"))
        .unwrap()
        .replace(
            "path = \"../../../../",
            &format!("path = \"{}/", repo().display()),
        );
    std::fs::write(c.path().join("Cargo.toml"), edit_toml(cargo_toml)).unwrap();
    let lib_rs = std::fs::read_to_string(fixture.join("src/lib.rs")).unwrap();
    std::fs::write(c.path().join("src/lib.rs"), edit(lib_rs)).unwrap();
    let snapshots = c.path().join(SNAPSHOTS_DIR);
    std::fs::create_dir_all(&snapshots).unwrap();
    for entry in std::fs::read_dir(fixture.join(SNAPSHOTS_DIR)).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, snapshots.join(path.file_name().unwrap())).unwrap();
    }
}

/// The fixture's source against its saved snapshot keeps the wire (EVO-70): a definition it embeds from the live
/// stdlib may have changed since its pins were refreshed, which is fine while it stays compatible.
fn assert_keeps_wire(outcome: &Outcome) -> &Report {
    let Outcome::Compared(comparison) = outcome else {
        panic!("{outcome:?}")
    };
    assert!(
        pins::wire_compatible(&comparison.report),
        "{outcome}\nThe nucleo_g0b1re_api fixture (deployed firmware's API 0.4.0) no longer works with its saved \
         snapshot on the wire: a stdlib definition it embeds changed incompatibly, see `breaking` above. If the \
         break is intended (a stated wire break, with its EVO item and CHANGELOG entry), refresh the fixture's pins \
         with `just refresh-fixture-pins --accept-breaking`; never edit the signatures by hand."
    );
    &comparison.report
}

#[test]
fn old_format_snapshot_passes_against_unchanged_source() {
    // the fixture as it is: whatever the stdlib changed since its pins were refreshed keeps the wire
    let fixture = old_format_fixture();
    let outcome = Check::new(&fixture)
        .against(old_format_snapshot(&fixture))
        .run()
        .unwrap();
    assert_keeps_wire(&outcome);

    // upgrading wire_weaver alone is no change of the API (EVO-13): the snapshot has no future_version, which codegen
    // now records for the crate's SiliconRevision. Checked on a copy with its pins refreshed to today's stdlib, so
    // that a compatible stdlib change doesn't show up here.
    let c = TestCrate::new("old-format-unchanged");
    copy_fixture(&c, |lib_rs| lib_rs);
    let cache = CrateCache::default();
    pins::refresh(c.path(), None, false, false, &cache).unwrap();
    let outcome = Check::new(c.path())
        .require_current(true)
        .cache(cache.clone())
        .run()
        .unwrap();
    outcome
        .verdict()
        .unwrap_or_else(|e| panic!("{outcome}\n{e}"));

    let outcome = check_fixture(&c, &cache);
    assert_eq!(outcome.verdict().unwrap(), "no changes", "{outcome}");
    // ItemStruct::extra (DER-18 transparent, INT-13 marker) came later still, then ApiBundle::wire (EVO-24),
    // ApiBundle::ext_wire (EVO-26), ApiItem::identity (INT-17), Meta::effects (API-58, model format 0.5.0) and
    // Meta::track (API-71, model format 0.6.0)
    assert_eq!(
        unrecorded(&outcome),
        [
            "ApiBundle::ext_wire",
            "ApiBundle::wire",
            "ApiItem::identity",
            "ItemEnum::future_version",
            "ItemStruct::extra",
            "Meta::effects",
            "Meta::track"
        ]
    );
    has_all(
        &outcome.to_string(),
        &[
            "not compared: ApiBundle::ext_wire, ApiBundle::wire, ApiItem::identity, ItemEnum::future_version, ItemStruct::extra, Meta::effects, Meta::track, \
            which a snapshot saved by an older wire_weaver",
        ],
    );
}

/// The fixture with `ww_uid`, a stdlib crate it embeds (`Device::uid`), replaced by a copy of the same version whose
/// `src/lib.rs` is changed by `edit`: a stdlib change in place, as a session makes one.
fn fixture_with_changed_uid(name: &str, edit: impl Fn(String) -> String) -> TestCrate {
    let c = TestCrate::new(name);
    let uid = c.path().join("ww_uid");
    std::fs::create_dir_all(uid.join("src")).unwrap();
    let source = repo().join("ww_uid");
    let lib_rs = std::fs::read_to_string(source.join("src/lib.rs")).unwrap();
    let changed = edit(lib_rs.clone());
    assert_ne!(changed, lib_rs, "the edit changed nothing");
    std::fs::write(uid.join("src/lib.rs"), changed).unwrap();
    // its own manifest, without the workspace's inheritance; ww_api from wire_weaver as Cargo.lock pins it
    std::fs::write(
        uid.join("Cargo.toml"),
        "[package]\nname = \"ww_uid\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n\
         [package.metadata.wire_weaver]\nwire = \"w1.0\"\n\n[features]\nstd = [\"ww_api/std\"]\n\
         defmt = [\"ww_api/defmt\"]\n\n[dependencies]\n\
         ww_api = { git = \"https://github.com/vhrdtech/wire_weaver\", default-features = false }\n",
    )
    .unwrap();
    let ww_uid = format!("path = \"{}\"", source.display());
    copy_fixture_with(
        &c,
        |lib_rs| lib_rs,
        |cargo_toml| {
            assert!(cargo_toml.contains(&ww_uid), "{cargo_toml}");
            cargo_toml.replace(&ww_uid, &format!("path = \"{}\"", uid.display()))
        },
    );
    c
}

const UID_VALUE: &str = "    pub value: MaxLen<RefVec<'i, u8>, 16>,";

/// [Check] of `c` against its old-format snapshot, through `cache`.
fn check_fixture(c: &TestCrate, cache: &CrateCache) -> Outcome {
    Check::new(c.path())
        .against(old_format_snapshot(c.path()))
        .cache(cache.clone())
        .run()
        .unwrap()
}

/// A compatible stdlib change in place (the id's max length raised) passes the fixture's check, and
/// `refresh-fixture-pins` takes it: only the pin's bytes change, and the fixture then compares clean (EVO-70). Before,
/// any change of an embedded stdlib definition read as the same "definitions changed" against the old-format
/// snapshot, and sessions edited the pinned signatures by hand.
#[test]
fn compatible_stdlib_change_passes_the_fixture() {
    let c = fixture_with_changed_uid("old-format-uid-compatible", |lib_rs| {
        lib_rs.replace(UID_VALUE, "    pub value: MaxLen<RefVec<'i, u8>, 32>,")
    });
    let cache = CrateCache::default();
    let snapshot = old_format_snapshot(c.path());
    let outcome = check_fixture(&c, &cache);
    let report = assert_keeps_wire(&outcome);
    assert_eq!(report.change, Change::Compatible, "{outcome}");
    has_all(&outcome.to_string(), &["Device::uid: max length raised"]);

    let saved = std::fs::read_to_string(&snapshot).unwrap();
    let refreshed = pins::refresh(c.path(), Some(&snapshot), false, false, &cache).unwrap();
    let what: Vec<_> = refreshed
        .refreshed
        .iter()
        .map(|r| r.what.as_str())
        .collect();
    assert_eq!(what, ["trait ww_uid::Uid"], "{:?}", refreshed.refreshed);
    let text = std::fs::read_to_string(&snapshot).unwrap();
    assert_ne!(text, saved);
    // only the pin's bytes changed, the old format is kept
    assert_eq!(text.lines().count(), saved.lines().count());
    assert!(!text.contains("future_version"));
    let outcome = check_fixture(&c, &cache);
    assert_eq!(outcome.verdict().unwrap(), "no changes", "{outcome}");
}

/// An incompatible stdlib change in place (the id's bytes as a `u32`) fails the fixture's check, named at the field,
/// and `refresh-fixture-pins` refuses it unless the break is accepted on purpose (EVO-70).
#[test]
fn breaking_stdlib_change_fails_the_fixture() {
    let c = fixture_with_changed_uid("old-format-uid-breaking", |lib_rs| {
        lib_rs.replace(UID_VALUE, "    pub value: u32,")
    });
    let cache = CrateCache::default();
    let snapshot = old_format_snapshot(c.path());
    let outcome = check_fixture(&c, &cache);
    let Outcome::Compared(comparison) = &outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(comparison.report.change, Change::Breaking, "{outcome}");
    assert!(!pins::wire_compatible(&comparison.report));
    let text = outcome.to_string();
    has_all(
        &text,
        &["trait Uid: IDS: struct UniqueId: field `value`: u32 on the new version"],
    );
    // no "doc comments changed, the definition is the same" note for a definition that changed
    assert!(!text.contains("the definition is the same"), "{text}");

    let saved = std::fs::read_to_string(&snapshot).unwrap();
    let err = pins::refresh(c.path(), Some(&snapshot), false, false, &cache).unwrap_err();
    has_all(&err, &["field `value`", "--accept-breaking"]);
    assert_eq!(std::fs::read_to_string(&snapshot).unwrap(), saved);
    // accepted on purpose: refreshed, and the fixture then compares clean
    pins::refresh(c.path(), Some(&snapshot), true, false, &cache).unwrap();
    let outcome = check_fixture(&c, &cache);
    assert_eq!(outcome.verdict().unwrap(), "no changes", "{outcome}");
}

#[test]
fn real_change_against_old_format_snapshot_is_caught() {
    let c = TestCrate::new("old-format-change");
    copy_fixture(&c, |lib_rs| {
        lib_rs.replace("    #[effect(actuate)]\n    fn led_off();\n", "")
    });
    let (outcome, verdict) = check_current(c.path());
    let Outcome::Compared(comparison) = &outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(comparison.report.change, Change::Breaking, "{outcome}");
    has_all(
        &verdict.unwrap_err(),
        &["0.5.0 or later", "The source differs from"],
    );

    // and a compatible one: a variant appended to the enum with the catch-all, which the old snapshot can't tell
    // had one, so the bump is breaking as for an enum without (docs/evolution/rules.md#unsized-types)
    let c = TestCrate::new("old-format-variant");
    copy_fixture(&c, |lib_rs| {
        lib_rs.replace(
            "    Unknown,\n}",
            "    Unknown,\n    /// Revision B\n    B,\n}",
        )
    });
    let (outcome, verdict) = check_current(c.path());
    assert!(verdict.is_err(), "{outcome}");
    let Outcome::Compared(comparison) = &outcome else {
        panic!("{outcome:?}")
    };
    assert_ne!(comparison.report.change, Change::None, "{outcome}");
}

/// The pin of `ww_timebase::Clock` that wire_weaver's copy of `nucleo_g0b1re_api_0_4_0.ron` holds: its
/// `tests/stdlib_evolution` reads that one pin for its check that a refresh in place keeps a pinned signature known.
const CLOCK_PIN: [u8; 8] = [99, 64, 185, 25, 39, 228, 153, 130];

/// The fixture's `ww_timebase::Clock` pin is the one wire_weaver's copy of the 0.4.0 snapshot holds. A refresh that
/// changes it (`just refresh-fixture-pins` after a `ww_timebase` change) fails here, until the refreshed file is copied
/// over wire_weaver's and [CLOCK_PIN] follows.
#[test]
fn clock_pin_matches_wire_weavers_copy() {
    let path = old_format_snapshot(&old_format_fixture());
    let bundle = snapshot_files::read(&path).unwrap().bundle;
    let pins: Vec<&[u8]> = bundle
        .traits
        .iter()
        .filter_map(|location| match location {
            ApiLevelLocationOwned::SkippedFullVersion {
                crate_idx,
                trait_name,
                signature,
            } if trait_name == "Clock" => {
                let crate_id = &bundle.crate_version(crate_idx.0).unwrap().crate_id;
                assert_eq!(crate_id, "ww_timebase", "{}", path.display());
                Some(signature.as_slice())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        pins,
        [CLOCK_PIN.as_slice()],
        "{}: the pin of ww_timebase::Clock changed. wire_weaver's tests/stdlib_evolution/pins/\
         nucleo_g0b1re_api_0_4_0.ron is a copy of this file for that pin: copy the refreshed file over it in \
         wire_weaver, then set CLOCK_PIN to the new pin.",
        path.display()
    );
}
