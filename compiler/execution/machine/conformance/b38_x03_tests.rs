//! Admission/inventory only. No execution, sessions, native code or new authority.
#![allow(unexpected_cfgs)]
#[cfg(b38_x03)]
mod investigation {
    use super::super::*;
    use std::io::{self, Write};

    struct BoundedFile {
        file: std::fs::File,
        left: usize,
    }
    impl Write for BoundedFile {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > self.left {
                return Err(io::Error::other("X03 output ceiling"));
            }
            let count = self.file.write(bytes)?;
            self.left -= count;
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.file.flush()
        }
    }
    fn limits() -> AdmissionLimits {
        AdmissionLimits {
            source_bytes: 65_536,
            tokens: 4096,
            identifier_bytes: 128,
            frontend_work: 10_000_000_000_000_000,
            traversal_depth: 22,
            alias_depth: 48,
            alias_work: 1_000_000,
            working_storage: 1_073_741_824,
            diagnostic_storage: 4_194_304,
            rendered_output: 65_536,
        }
    }
    fn inventory(
        out: &mut BoundedFile,
        label: &str,
        admitted: Result<CheckedExecutable, ExecutionError>,
        roots: &[(&str, &[&str])],
    ) -> io::Result<()> {
        writeln!(out, "FIXTURE {label}")?;
        let executable = match admitted {
            Ok(value) => value,
            Err(error) => {
                writeln!(out, "ADMISSION_FAILURE {error:#?}")?;
                return Err(io::Error::other("source rejected"));
            }
        };
        writeln!(out, "ADMISSION_OK\nHIR {:#?}", executable.program)?;
        writeln!(out, "MIR {:#?}", executable.mir)?;
        writeln!(out, "ORIGINS {:#?}", executable.origins)?;
        writeln!(out, "TRUSTED {:#?}", executable.trusted)?;
        writeln!(out, "OUTCOMES {:#?}", executable.outcomes)?;
        writeln!(out, "CLEANUP {:#?}", executable.cleanup_plans)?;
        writeln!(out, "CLEANUP_EFFECTS {:#?}", executable.cleanup_effects)?;
        writeln!(out, "RESULT_PLANS {:#?}", executable.result_plans)?;
        for (root, expected) in roots {
            writeln!(out, "STAGE {label} root={root} extraction-start")?;
            let entry = executable.entry(root).expect("admitted root");
            let contract = match contract::Contract::extract(
                &executable,
                entry,
                contract::Limits::default(),
            ) {
                Ok(value) => value,
                Err(error) => {
                    writeln!(out, "EXTRACTION_FAILURE root={root} {error:#?}")?;
                    return Err(io::Error::other("contract extraction"));
                }
            };
            if let Err(error) = contract.validate() {
                writeln!(out, "VALIDATION_FAILURE root={root} {error:#?}")?;
                return Err(io::Error::other("unchanged contract validation"));
            }
            writeln!(out, "STAGE {label} root={root} extraction-validated")?;
            let mut names: Vec<_> = contract
                .functions()
                .iter()
                .map(|id| executable.program.function(*id).name())
                .collect();
            names.sort_unstable();
            let mut wanted = expected.to_vec();
            wanted.sort_unstable();
            writeln!(out, "ROOT {root} ENTRY {entry:?}")?;
            writeln!(out, "CLOSURE {:?}", contract.functions())?;
            writeln!(out, "SCHEMA {:?}", contract.schema())?;
            writeln!(out, "SCHEDULE {:?}", contract.schedule())?;
            writeln!(out, "SITES {:#?}", contract.sites())?;
            writeln!(out, "OPERANDS {:#?}", contract.expressions())?;
            assert_eq!(names, wanted, "source-derived structural closure");
        }
        Ok(())
    }
    #[test]
    fn b38_x03_admission_inventory_only() -> io::Result<()> {
        crate::execution::reset_execution_observations();
        let path = std::env::var_os("B38_X03_OUTPUT").expect("output path");
        let directory = std::path::PathBuf::from(path);
        assert!(directory.is_absolute(), "task-owned absolute output");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("inventory.txt"))?;
        let mut out = BoundedFile {
            file,
            left: 8 * 1024 * 1024,
        };
        writeln!(out, "B38-X03 static-v1\nADMISSION_LIMITS {:?}", limits())?;
        let completion_geometry = (
            std::mem::size_of::<crate::trusted::approval::CompletionPlan>(),
            std::mem::size_of::<crate::trusted::approval::Invocation>(),
        );
        writeln!(
            out,
            "BUILD_GEOMETRY CompletionPlan,Invocation={completion_geometry:?} unit=bytes"
        )?;
        writeln!(
            out,
            "SCHEDULE_UNITS representation/capacity=bytes fuel_unit=logical-actions"
        )?;
        for labels in [
            "machine=Frame,Task,Value,PlaceSlot,FieldValue,FieldRange,CallTrace,ResultLayer",
            concat!(
                "cleanup=State,Event,ResourceFrame,Activation,Generation,Pending,",
                "LocalGen,FieldValue,ReplacementPrepared"
            ),
            "results=State,Cell,OwnerPair,InertRecord,ResultLayer",
            "temporaries=State,Activation,Owner,Event",
            "trusted=State,Entry,Production,Event,Sensor,Receipt,CompletionPending,Payload",
            "observation=Record,RefCellOptionCapture",
            "comparison=Node,Link,SlotRowPlusHistory,Pair,TrustedEvent,LeafPair,Scratch,Comparison",
            "extraction=Site,FunctionId,IndexPair,Contract",
        ] {
            writeln!(out, "SCHEDULE_INDEX_ORDER {labels} unit=bytes")?;
        }
        let comp = include_str!("../../../../experiments/bed-038/x03/computational.bow");
        assert_eq!(comp.len(), 947);
        let roots: &[(&str, &[&str])] = &[
            ("compute", &["compute", "seed", "step"]),
            ("trap", &["trap", "seed", "step"]),
            ("short", &["short", "step"]),
            ("wide_trap", &["wide_trap", "wide_seed"]),
        ];
        writeln!(out, "STAGE computational admission-start")?;
        inventory(
            &mut out,
            "computational",
            prepare_execution(comp, limits()),
            roots,
        )?;
        let combined = concat!(
            include_str!("../../../../examples/sentinel.bow"),
            "\n",
            include_str!("../../../../experiments/bed-038/x03/companion-suffix.bow")
        );
        assert_eq!(combined.len(), 7866);
        let selection = crate::trusted::Selection::sensor_demo(
            crate::trusted::Scenario::Sample,
            crate::trusted::Grants::ALL,
            &["monitor", "b38_probe"],
        )
        .expect("selection")
        .with_sensor_limit(1)
        .expect("live limit");
        writeln!(out, "SELECTION {selection:?}")?;
        let roots: &[(&str, &[&str])] = &[
            ("monitor", &["monitor", "failed", "read_code"]),
            ("b38_probe", &["b38_probe", "b38_read", "b38_relay"]),
        ];
        writeln!(out, "STAGE sentinel-companion admission-start")?;
        let admitted = prepare_execution_with_profile(combined, limits(), &selection);
        inventory(&mut out, "sentinel-companion", admitted, roots)?;
        let (_, starts) = crate::execution::execution_observations();
        writeln!(out, "MACHINE_STARTS {starts}\nEND static-v1")?;
        assert_eq!(starts, 0, "inventory must never execute the interpreter");
        out.flush()
    }
}
