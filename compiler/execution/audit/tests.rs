use super::*;
use crate::trusted::{Grants, Scenario, Selection};
const SOURCE:&str="fn flag()->bool{return true;}fn helper()->bool{return flag();}fn unused()->Result<success:Reading,failure:SensorError>effects io{let sensor=sensor_open()or return;return sensor_read(&sensor);}fn demo()->bool{return helper();}";
fn artifact(source: &str) -> CheckedExecutable {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    prepare_execution_with_profile(source, AdmissionLimits::default(), &selection).unwrap()
}
#[test]
fn audit_is_deterministic_and_distinguishes_unused_grants_from_rooted_use() {
    let e = artifact(SOURCE);
    let text = e.render_authority(ExecutionLimits::default()).unwrap();
    assert_eq!(
        text,
        e.render_authority(ExecutionLimits::default()).unwrap()
    );
    assert!(text.contains("root: function=3 name=demo"));
    assert!(text.contains("granted=true; structurally-used=false"));
    assert!(!text.contains("trusted site:"));
    assert!(text.contains("reachable edge: caller=3 callee=Source(FunctionId(1))"));
    assert!(text.contains("reachable edge: caller=1 callee=Source(FunctionId(0))"));
    for required in [
        "assumptions=",
        "scenario=Sample",
        "live-sensor-limit=16",
        "cleanup-only-release=",
        "release-granted=true",
        "release-binding=SensorReleaseV1",
        "Reading=nominal range 0..4095",
        "Error={2:no sample",
        "protected Results",
        "teardown is not successful",
    ] {
        assert!(text.contains(required), "{required}");
    }
}
#[test]
fn structural_audit_includes_both_alternatives_zero_effect_helpers_and_cleanup_sites() {
    let source="fn flag()->bool{return true;}fn helper()->bool{return flag();}fn demo()->Result<success:Reading,failure:SensorError>effects io{let sensor=sensor_open()or return;if helper(){return sensor_read(&sensor);}else{return sensor_read(&sensor);}}";
    let e = artifact(source);
    let text = e.render_authority(ExecutionLimits::default()).unwrap();
    assert_eq!(text.matches("trusted site:").count(), 3);
    assert!(text.contains("granted=true; structurally-used=true"));
    assert!(text.contains("possible cleanup:"));
    assert!(text.contains("possible temporary cleanup:"));
    assert!(text.contains("callee=Source(FunctionId(0))"));
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    assert!(s.events().is_empty());
    s.advance().unwrap();
    assert_eq!(
        text,
        s.render_authority(ExecutionLimits::default()).unwrap()
    );
}
#[test]
fn audit_exact_output_storage_and_work_neighbors_are_bounded() {
    let e = artifact(SOURCE);
    let normal = e.render_authority(ExecutionLimits::default()).unwrap();
    // M4 changes only these two live prose spellings. Each adds one actual byte;
    // retained historical measurements are not rewritten to hide that difference.
    for (legacy, current) in [
        ("Klyxr-visible Sensor state", "Bowfin-visible Sensor state"),
        ("successful Klyxr cleanup", "successful Bowfin cleanup"),
    ] {
        assert_eq!(normal.matches(current).count(), 1);
        assert!(!normal.contains(legacy));
        assert_eq!(current.len(), legacy.len() + 1);
    }
    let n = normal.len();
    let limits = ExecutionLimits {
        rendered_output: n,
        ..ExecutionLimits::default()
    };
    assert_eq!(e.render_authority(limits).unwrap(), normal);
    assert_eq!(
        e.render_authority(ExecutionLimits {
            rendered_output: n - 1,
            ..limits
        })
        .unwrap_err()
        .kind,
        ErrorKind::ConfiguredLimit("rendered output")
    );
    let bytes = e.program.functions().len() + std::mem::size_of::<render::Output>() + n;
    assert!(e
        .render_authority(ExecutionLimits {
            working_storage: bytes,
            ..limits
        })
        .is_ok());
    assert_eq!(
        e.render_authority(ExecutionLimits {
            working_storage: bytes - 1,
            ..limits
        })
        .unwrap_err()
        .kind,
        ErrorKind::ConfiguredLimit("authority audit storage")
    );
    let mut low = 1;
    let mut high = ExecutionLimits::default().steps;
    while low < high {
        let mid = (low + high) / 2;
        if e.render_authority(ExecutionLimits {
            steps: mid,
            ..limits
        })
        .is_ok()
        {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    assert!(e
        .render_authority(ExecutionLimits {
            steps: low,
            ..limits
        })
        .is_ok());
    assert_eq!(
        e.render_authority(ExecutionLimits {
            steps: low - 1,
            ..limits
        })
        .unwrap_err()
        .kind,
        ErrorKind::ConfiguredLimit("authority audit work")
    );
}
#[test]
fn actual_reachability_and_renderer_allocation_failures_stay_host() {
    let e = artifact(SOURCE);
    FAIL_SCRATCH.with(|f| f.set(true));
    let result = e.render_authority(ExecutionLimits::default());
    FAIL_SCRATCH.with(|f| f.set(false));
    assert_eq!(
        result.unwrap_err().kind,
        ErrorKind::Host("authority reachability allocation")
    );
    assert_eq!(
        render::with_allocation_failure(|| e.render_authority(ExecutionLimits::default()))
            .unwrap_err()
            .kind,
        ErrorKind::Host("render allocation")
    );
}
#[test]
fn no_profile_and_mutated_approval_cannot_gain_authority_from_audit() {
    let e = prepare_execution("fn demo(){}", AdmissionLimits::default()).unwrap();
    assert!(matches!(
        e.render_authority(ExecutionLimits::default())
            .unwrap_err()
            .kind,
        ErrorKind::Integrity(_)
    ));
    let mut e = artifact(SOURCE);
    e.trusted.as_mut().unwrap().roots.clear();
    assert!(matches!(
        e.render_authority(ExecutionLimits::default())
            .unwrap_err()
            .kind,
        ErrorKind::Integrity(_)
    ));
}
