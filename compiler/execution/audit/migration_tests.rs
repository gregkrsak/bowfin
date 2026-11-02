//! M1 checks the existing sealed producer/validator/renderer, not an importer.
//! The legacy strings below are inert, exact fixtures. They create no aliases,
//! approval, grants, catalog origins, custody, schema validity or replay identity.
use super::*;
use crate::execution::contract::{Contract, Limits};
use crate::trusted::{approval::Approved, Grants, Scenario, Selection};

const SOURCE: &str = "fn demo()->bool{return true;}fn unused()->bool{return false;}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    Target,
    Compiler,
    Catalog,
    Profile,
    Assumptions,
}

// These exact 0.0.1 spellings record the M1 correspondence, not an open-ended
// version matcher. The legacy side remains unchanged historical test data.
const MAPPINGS: [(Field, &str, &str); 5] = [
    (
        Field::Target,
        "klyxr.ordinary-mir.v1",
        "bowfin.ordinary-mir.v1",
    ),
    (
        Field::Compiler,
        "klyxr-compiler/0.0.1",
        "bowfin-compiler/0.0.1",
    ),
    (
        Field::Catalog,
        "klyxr.sensor-demo.catalog.v1",
        "bowfin.sensor-demo.catalog.v1",
    ),
    (
        Field::Profile,
        "klyxr.sensor-demo.profile.v1",
        "bowfin.sensor-demo.profile.v1",
    ),
    (
        Field::Assumptions,
        "klyxr.sensor-demo.assumptions.v1",
        "bowfin.sensor-demo.assumptions.v1",
    ),
];

const LEGACY_HEADER: &str = concat!(
    "authority account v1\n",
    "target=klyxr.ordinary-mir.v1\n",
    "compiler=klyxr-compiler/0.0.1\n",
    "catalog=klyxr.sensor-demo.catalog.v1\n",
    "profile=klyxr.sensor-demo.profile.v1\n",
    "assumptions=klyxr.sensor-demo.assumptions.v1\n",
);
const CURRENT_HEADER: &str = concat!(
    "authority account v1\n",
    "target=bowfin.ordinary-mir.v1\n",
    "compiler=bowfin-compiler/0.0.1\n",
    "catalog=bowfin.sensor-demo.catalog.v1\n",
    "profile=bowfin.sensor-demo.profile.v1\n",
    "assumptions=bowfin.sensor-demo.assumptions.v1\n",
);

fn selection(grants: Grants) -> Selection {
    Selection::sensor_demo(Scenario::Sample, grants, &["demo"]).unwrap()
}

fn artifact() -> CheckedExecutable {
    prepare_execution_with_profile(SOURCE, AdmissionLimits::default(), &selection(Grants::ALL))
        .unwrap()
}

fn identity(approved: &Approved, field: Field) -> &'static str {
    match field {
        Field::Target => approved.target,
        Field::Compiler => approved.compiler,
        Field::Catalog => approved.catalog_identity,
        Field::Profile => approved.profile_identity,
        Field::Assumptions => approved.assumptions,
    }
}

fn inject(executable: &mut CheckedExecutable, field: Field, value: &'static str) {
    let approved = executable.trusted.as_mut().unwrap();
    match field {
        Field::Target => approved.target = value,
        Field::Compiler => approved.compiler = value,
        Field::Catalog => approved.catalog_identity = value,
        Field::Profile => approved.profile_identity = value,
        Field::Assumptions => approved.assumptions = value,
    }
}

fn identity_rejects(executable: &CheckedExecutable) {
    let expected = ErrorKind::Integrity("trusted target/compiler/catalog/profile identity");
    assert_eq!(
        executable
            .trusted
            .as_ref()
            .unwrap()
            .shape(AdmissionLimits::default())
            .unwrap_err(),
        crate::hir::resources::PlanError::Integrity(
            "trusted target/compiler/catalog/profile identity",
        )
    );
    assert_eq!(executable.validate().unwrap_err().kind, expected);
    assert_eq!(
        executable
            .render_authority(ExecutionLimits::default())
            .unwrap_err()
            .kind,
        expected
    );
}

#[test]
fn m1_machine_identity_table_is_exact_injective_and_accounts_for_output_bytes() {
    let executable = artifact();
    let approved = executable.trusted.as_ref().unwrap();
    for (index, (field, legacy, current)) in MAPPINGS.iter().enumerate() {
        assert_eq!(identity(approved, *field), *current, "{field:?}");
        assert_ne!(legacy, current);
        assert_eq!(current.len(), legacy.len() + 1);
        for (other_field, other_legacy, other_current) in &MAPPINGS[index + 1..] {
            assert_ne!(field, other_field);
            assert_ne!(legacy, other_legacy);
            assert_ne!(current, other_current);
        }
        for (_, _, other_current) in MAPPINGS {
            assert_ne!(*legacy, other_current);
        }
    }
    let output = executable
        .render_authority(ExecutionLimits::default())
        .unwrap();
    assert!(output.starts_with(CURRENT_HEADER));
    assert!(!output.starts_with(LEGACY_HEADER));
    assert_eq!(CURRENT_HEADER.len(), LEGACY_HEADER.len() + 5);

    // The larger output still consumes actual bytes. No limit is silently raised
    // to make the renamed account fit its predecessor's measured boundary.
    let exact = ExecutionLimits {
        rendered_output: output.len(),
        ..ExecutionLimits::default()
    };
    assert_eq!(executable.render_authority(exact).unwrap(), output);
    assert_eq!(
        executable
            .render_authority(ExecutionLimits {
                rendered_output: output.len() - 1,
                ..exact
            })
            .unwrap_err()
            .kind,
        ErrorKind::ConfiguredLimit("rendered output")
    );
}

#[test]
fn m1_all_nonempty_legacy_machine_identity_mixes_reject() {
    // Every nonempty subset includes both the fully historical tuple and all
    // thirty mixed tuples; no spelling is normalized into current authority.
    let mut rejected = 0;
    for mask in 1usize..(1usize << MAPPINGS.len()) {
        let mut executable = artifact();
        for (index, (field, legacy, current)) in MAPPINGS.iter().enumerate() {
            if mask & (1usize << index) != 0 {
                inject(&mut executable, *field, legacy);
            } else {
                assert_eq!(
                    identity(executable.trusted.as_ref().unwrap(), *field),
                    *current
                );
            }
        }
        identity_rejects(&executable);
        rejected += 1;
    }
    assert_eq!(rejected, 31);
}

#[test]
fn m1_unknown_wrong_role_and_neighbor_machine_identities_reject() {
    const NEIGHBORS: [[&str; 4]; 5] = [
        [
            "bowfin.unknown-target.v1",
            "Bowfin.ordinary-mir.v1",
            "bowfin.ordinary-mir.v2",
            "bowfin.ordinary-mir.v10",
        ],
        [
            "bowfin-compiler/unknown",
            "Bowfin-compiler/0.0.1",
            "bowfin-compiler/0.0.2",
            "bowfin-compiler/0.0.10",
        ],
        [
            "bowfin.sensor-demo.unknown-catalog.v1",
            "Bowfin.sensor-demo.catalog.v1",
            "bowfin.sensor-demo.catalog.v2",
            "bowfin.sensor-demo.catalog.v10",
        ],
        [
            "bowfin.sensor-demo.unknown-profile.v1",
            "Bowfin.sensor-demo.profile.v1",
            "bowfin.sensor-demo.profile.v2",
            "bowfin.sensor-demo.profile.v10",
        ],
        [
            "bowfin.sensor-demo.unknown-assumptions.v1",
            "Bowfin.sensor-demo.assumptions.v1",
            "bowfin.sensor-demo.assumptions.v2",
            "bowfin.sensor-demo.assumptions.v10",
        ],
    ];
    for (index, (field, _, _)) in MAPPINGS.iter().enumerate() {
        // A recognized identity in a different field is not an alias either.
        for (other_field, other_legacy, other_current) in MAPPINGS {
            if *field != other_field {
                for value in [other_legacy, other_current] {
                    let mut executable = artifact();
                    inject(&mut executable, *field, value);
                    identity_rejects(&executable);
                }
            }
        }
        for value in NEIGHBORS[index] {
            let mut executable = artifact();
            inject(&mut executable, *field, value);
            identity_rejects(&executable);
        }
    }
}

#[test]
fn m1_machine_names_cannot_substitute_original_opaque_approval() {
    let first = artifact();
    let independent = artifact();
    for (field, _, current) in MAPPINGS {
        assert_eq!(identity(first.trusted.as_ref().unwrap(), field), current);
        assert_eq!(
            identity(independent.trusted.as_ref().unwrap(), field),
            current
        );
    }
    let first_contract =
        Contract::extract(&first, first.entry("demo").unwrap(), Limits::default()).unwrap();
    let second_view =
        Contract::extract(&first, first.entry("demo").unwrap(), Limits::default()).unwrap();
    let independent_contract = Contract::extract(
        &independent,
        independent.entry("demo").unwrap(),
        Limits::default(),
    )
    .unwrap();
    first_contract.validate().unwrap();
    second_view.validate().unwrap();
    independent_contract.validate().unwrap();
    assert_eq!(first_contract.schema(), independent_contract.schema());
    assert!(first_contract.same_approval(&second_view));
    assert!(!first_contract.same_approval(&independent_contract));
}

#[test]
fn m1_canonical_machine_names_grant_no_operations_or_extra_roots() {
    let executable = prepare_execution_with_profile(
        SOURCE,
        AdmissionLimits::default(),
        &selection(Grants::NONE),
    )
    .unwrap();
    let approved = executable.trusted.as_ref().unwrap();
    for (field, _, current) in MAPPINGS {
        assert_eq!(identity(approved, field), current);
    }
    for operation in &approved.catalog.operations {
        assert!(!approved.catalog.granted(operation.id));
    }
    assert!(executable
        .render_authority(ExecutionLimits::default())
        .unwrap()
        .starts_with(CURRENT_HEADER));
    assert_eq!(
        executable.entry("unused").unwrap_err().kind,
        ErrorKind::Integrity("selected entry is outside the sealed trusted roots")
    );

    let release_denied = Grants {
        release: false,
        ..Grants::ALL
    };
    for (source, grants, expected) in [
        (
            "fn demo()->bool{let unused=sensor_open();return true;}",
            Grants::NONE,
            "trusted operation `sensor_open` is not granted",
        ),
        (
            "fn read(sensor:&Sensor)->Result<success:Reading,failure:SensorError>effects io{return sensor_read(sensor);}fn demo(){}",
            Grants::NONE,
            "trusted operation `sensor_read` is not granted",
        ),
        (
            "fn finish(sensor:Sensor)->Result<success:FinishReceipt,failure:SensorError>effects io{return sensor_finish(sensor);}fn demo(){}",
            Grants::NONE,
            "trusted operation `sensor_finish` is not granted",
        ),
        (
            "fn demo()->bool{let unused=sensor_open();return true;}",
            release_denied,
            "trusted operation `sensor_open` is not granted",
        ),
    ] {
        let error = prepare_execution_with_profile(
            source,
            AdmissionLimits::default(),
            &selection(grants),
        )
        .err()
        .expect("canonical names must not supply the missing grant");
        match error.kind {
            ErrorKind::Frontend { message, .. } => assert_eq!(message, expected),
            other => panic!("expected the missing-grant diagnostic, got {other:?}"),
        }
    }
}
