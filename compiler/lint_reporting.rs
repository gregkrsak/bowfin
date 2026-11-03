//! BED-037's private CLI reporting account. No semantic or execution authority.
use crate::source_facts::SourceFacts;
use std::io::Write;

pub(crate) const DEFAULT_REPORT_BYTES: usize = 65_536;
pub(crate) const MAX_REPORT_BYTES: usize = 1_048_576;
const WARNING: &[u8] =
    b"warning[commentless-source]: this source file contains no comments\n  --> ";
const DENIAL: &[u8] =
    b"error[commentless-source]: documentation policy denies a source file with no comments\n  --> ";
const HELP: &[u8] =
    b"\nhelp: explain the file's purpose, domain meaning, or important assumptions\n";
const LIMIT_EMERGENCY: &[u8] =
    b"\nerror: commentless-source reporting limit exhausted; command stopped\n";
const HOST_EMERGENCY: &[u8] =
    b"\nerror: commentless-source host reporting failure; command stopped\n";
const EMERGENCY_BYTES: usize = if LIMIT_EMERGENCY.len() > HOST_EMERGENCY.len() {
    LIMIT_EMERGENCY.len()
} else {
    HOST_EMERGENCY.len()
};
const DECIMAL_DIGITS: usize = usize::MAX.ilog10() as usize + 1;
const FRAME_BYTES: usize = DENIAL.len() + HELP.len() + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Allow,
    Warn,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settings {
    pub severity: Severity,
    pub report_bytes: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            severity: Severity::Warn,
            report_bytes: DEFAULT_REPORT_BYTES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    NoFinding,
    Suppressed,
    ReportedWarning,
    ReportedDenial,
    ReportingLimit,
    HostReportingFailure,
}

impl Outcome {
    pub(crate) fn exit_code(self) -> Option<i32> {
        match self {
            Self::NoFinding | Self::Suppressed | Self::ReportedWarning => None,
            Self::ReportedDenial | Self::ReportingLimit => Some(1),
            Self::HostReportingFailure => Some(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Failure {
    Limit,
    Host,
}

/// Logical work is independent of every predecessor frontend/runtime account.
#[derive(Default)]
struct Work {
    path: usize,
    frame: usize,
    decimal: usize,
    appended: usize,
}

fn charge(value: &mut usize, amount: usize, ceiling: usize) -> Result<(), Failure> {
    *value = value.checked_add(amount).ok_or(Failure::Limit)?;
    if *value > ceiling {
        return Err(Failure::Limit);
    }
    Ok(())
}

trait Sink {
    fn append(&mut self, bytes: &[u8], work: &mut Work) -> Result<(), Failure>;
}

struct Sizing {
    length: usize,
    ceiling: usize,
}

impl Sink for Sizing {
    fn append(&mut self, bytes: &[u8], _work: &mut Work) -> Result<(), Failure> {
        charge(&mut self.length, bytes.len(), self.ceiling)
    }
}

struct Record {
    bytes: Vec<u8>,
    demand: usize,
}

impl Sink for Record {
    fn append(&mut self, bytes: &[u8], work: &mut Work) -> Result<(), Failure> {
        let length = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(Failure::Limit)?;
        if length > self.demand || self.bytes.capacity() < self.demand {
            return Err(Failure::Host);
        }
        charge(&mut work.appended, bytes.len(), self.demand)?;
        // Capacity was requested once, after sizing. This cannot grow the buffer.
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

fn frame(sink: &mut impl Sink, work: &mut Work, bytes: &[u8]) -> Result<(), Failure> {
    charge(&mut work.frame, bytes.len(), 2 * FRAME_BYTES)?;
    sink.append(bytes, work)
}

fn number(sink: &mut impl Sink, work: &mut Work, mut value: usize) -> Result<(), Failure> {
    let mut scratch = [0u8; DECIMAL_DIGITS];
    let mut start = scratch.len();
    loop {
        charge(&mut work.decimal, 1, 4 * DECIMAL_DIGITS)?;
        start -= 1;
        scratch[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            return sink.append(&scratch[start..], work);
        }
    }
}

fn record(
    sink: &mut impl Sink,
    work: &mut Work,
    path: &str,
    line: usize,
    column: usize,
    settings: Settings,
) -> Result<(), Failure> {
    frame(
        sink,
        work,
        if settings.severity == Severity::Deny {
            DENIAL
        } else {
            WARNING
        },
    )?;
    // Borrow the CLI's already accepted UTF-8 string; do not normalize or clone it.
    let path_ceiling = settings
        .report_bytes
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .ok_or(Failure::Limit)?;
    for &byte in path.as_bytes() {
        charge(&mut work.path, 1, path_ceiling)?;
        #[cfg(test)]
        trace(|t| t.path_bytes += 1);
        match byte {
            b'\\' => sink.append(b"\\\\", work)?,
            b'\n' => sink.append(b"\\n", work)?,
            b'\r' => sink.append(b"\\r", work)?,
            b'\t' => sink.append(b"\\t", work)?,
            0..=31 | 127 => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                sink.append(
                    &[
                        b'\\',
                        b'x',
                        HEX[(byte >> 4) as usize],
                        HEX[(byte & 15) as usize],
                    ],
                    work,
                )?;
            }
            _ => sink.append(&[byte], work)?,
        }
    }
    frame(sink, work, b":")?;
    number(sink, work, line)?;
    frame(sink, work, b":")?;
    number(sink, work, column)?;
    frame(sink, work, HELP)
}

/// Instrument the actual allocation request, not a separately predicted demand.
fn reserve_record_payload(bytes: &mut Vec<u8>, requested: usize) -> Result<(), Failure> {
    #[cfg(test)]
    {
        trace(|t| {
            t.allocations += 1;
            t.request = requested;
        });
        if fault(Fault::Allocation)
            || CONTROL.with(|t| {
                t.borrow()
                    .maximum_request
                    .is_some_and(|maximum| requested > maximum)
            })
        {
            return Err(Failure::Host);
        }
    }
    bytes
        .try_reserve_exact(requested)
        .map_err(|_| Failure::Host)
}

fn render(facts: &SourceFacts, path: &str, settings: Settings) -> Result<Record, Failure> {
    if settings.report_bytes == 0 || settings.report_bytes > MAX_REPORT_BYTES {
        return Err(Failure::Limit);
    }
    let span = facts.first_code_token().ok_or(Failure::Host)?;
    let mut work = Work::default();
    let mut sizing = Sizing {
        length: 0,
        ceiling: settings.report_bytes,
    };
    #[cfg(test)]
    {
        trace(|t| t.sizing_calls += 1);
        if fault(Fault::Sizing) {
            return Err(Failure::Limit);
        }
    }
    record(
        &mut sizing,
        &mut work,
        path,
        span.line,
        span.column,
        settings,
    )?;
    let mut result = Record {
        bytes: Vec::new(),
        demand: sizing.length,
    };
    reserve_record_payload(&mut result.bytes, sizing.length)?;
    #[cfg(test)]
    {
        trace(|t| t.render_calls += 1);
        if fault(Fault::Rendering) {
            return Err(Failure::Host);
        }
    }
    record(
        &mut result,
        &mut work,
        path,
        span.line,
        span.column,
        settings,
    )?;
    #[cfg(test)]
    if fault(Fault::Mismatch) {
        result.bytes.pop();
    }
    if result.bytes.len() != sizing.length {
        return Err(Failure::Host);
    }
    Ok(result)
}

/// Unlike write_all, an Interrupted error is terminal here. Every successful
/// call makes positive progress, followed by exactly one explicit final flush.
fn deliver(destination: &mut impl Write, bytes: &[u8], ceiling: usize) -> Result<(), Failure> {
    let mut written = 0usize;
    let mut calls = 0usize;
    let maximum_calls = ceiling.checked_add(1).ok_or(Failure::Limit)?;
    while written < bytes.len() {
        charge(&mut calls, 1, maximum_calls)?;
        let count = destination
            .write(&bytes[written..])
            .map_err(|_| Failure::Host)?;
        if count == 0 || count > bytes.len() - written {
            return Err(Failure::Host);
        }
        written += count;
    }
    charge(&mut calls, 1, maximum_calls)?;
    destination.flush().map_err(|_| Failure::Host)
}

pub(crate) fn report(
    facts: &SourceFacts,
    path: &str,
    settings: Settings,
    mut destination: impl Write,
) -> Outcome {
    if !facts.parsed_nonempty() || facts.has_comment() {
        return Outcome::NoFinding;
    }
    if settings.severity == Severity::Allow {
        return Outcome::Suppressed;
    }
    let failure = match render(facts, path, settings) {
        Ok(record) => match deliver(&mut destination, &record.bytes, settings.report_bytes) {
            Ok(()) => {
                return if settings.severity == Severity::Deny {
                    Outcome::ReportedDenial
                } else {
                    Outcome::ReportedWarning
                };
            }
            Err(error) => error,
        },
        Err(error) => error,
    };
    // The normal owned record is dropped before this borrowed emergency attempt.
    // Emergency success is never a successful warning or policy-error delivery.
    let emergency = deliver(
        &mut destination,
        match failure {
            Failure::Limit => LIMIT_EMERGENCY,
            Failure::Host => HOST_EMERGENCY,
        },
        EMERGENCY_BYTES,
    );
    if emergency == Err(Failure::Host) {
        return Outcome::HostReportingFailure;
    }
    match failure {
        Failure::Limit => Outcome::ReportingLimit,
        Failure::Host => Outcome::HostReportingFailure,
    }
}

/// The CLI and its fault-injection tests share this exact continuation. The
/// admitted real session stays alive, and cannot advance before complete lint
/// delivery. No session construction or admission is repeated here.
pub(crate) fn advance_selected<'s>(
    session: &'s mut crate::execution::ExecutionSession<'_>,
    facts: &SourceFacts,
    path: &str,
    settings: Settings,
    destination: impl Write,
) -> Result<
    Result<&'s crate::execution::ExecutionResult, &'s crate::execution::ExecutionError>,
    Outcome,
> {
    let outcome = report(facts, path, settings, destination);
    if outcome.exit_code().is_some() {
        return Err(outcome);
    }
    Ok(session.advance())
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fault {
    Sizing,
    Allocation,
    Rendering,
    Mismatch,
}

#[cfg(test)]
#[derive(Clone, Copy, Default)]
struct Trace {
    injected: Option<Fault>,
    maximum_request: Option<usize>,
    sizing_calls: usize,
    allocations: usize,
    request: usize,
    render_calls: usize,
    path_bytes: usize,
}

#[cfg(test)]
thread_local! {
    static CONTROL: std::cell::RefCell<Trace> = std::cell::RefCell::new(Trace::default());
}

#[cfg(test)]
fn trace(f: impl FnOnce(&mut Trace)) {
    CONTROL.with(|t| f(&mut t.borrow_mut()));
}

#[cfg(test)]
fn fault(expected: Fault) -> bool {
    CONTROL.with(|t| t.borrow().injected == Some(expected))
}

#[cfg(test)]
pub(crate) fn with_fault<T>(injected: Fault, f: impl FnOnce() -> T) -> T {
    struct Restore(Trace);
    impl Drop for Restore {
        fn drop(&mut self) {
            trace(|t| *t = self.0);
        }
    }
    let saved = CONTROL.with(|t| *t.borrow());
    let _restore = Restore(saved);
    trace(|t| {
        *t = Trace {
            injected: Some(injected),
            ..Trace::default()
        };
    });
    f()
}

#[cfg(test)]
pub(crate) fn fixed_layout_bytes() -> usize {
    // Conservative separate fixed-state ledger: all new state simultaneously
    // live along collection, sizing, rendering and delivery, including scalar
    // cursors, slices, formatting scratch and the owned buffer's fixed header.
    let wrapper =
        [
            std::mem::size_of::<crate::source_facts::SourcePrepared<crate::ast::Program>>()
                - std::mem::size_of::<crate::ast::Program>(),
            std::mem::size_of::<crate::source_facts::SourcePrepared<crate::hir::Program>>()
                - std::mem::size_of::<crate::hir::Program>(),
            std::mem::size_of::<
                crate::source_facts::SourcePrepared<crate::execution::CheckedExecutable>,
            >() - std::mem::size_of::<crate::execution::CheckedExecutable>(),
        ]
        .into_iter()
        .max()
        .unwrap();
    wrapper
        // The lexer collector plus the CLI's level-seen/capacity-seen flags.
        + 2 * std::mem::size_of::<bool>()
        + std::mem::size_of::<bool>()
        + std::mem::size_of::<SourceFacts>()
        + std::mem::size_of::<Settings>()
        + std::mem::size_of::<Outcome>()
        + std::mem::size_of::<Failure>()
        + std::mem::size_of::<Work>()
        + std::mem::size_of::<Sizing>()
        + std::mem::size_of::<Record>()
        + std::mem::size_of::<crate::lexer::Span>()
        + 4 * std::mem::size_of::<usize>()
        + 2 * std::mem::size_of::<&[u8]>()
        + DECIMAL_DIGITS
        + 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn facts(source: &str) -> SourceFacts {
        crate::parse_source_with_facts(source)
            .unwrap()
            .into_parts()
            .1
    }

    fn eligible() -> SourceFacts {
        facts("\n\t type Percent = range 0..100;")
    }

    fn expected() -> Vec<u8> {
        b"warning[commentless-source]: this source file contains no comments\n  --> charge.bow:2:3\nhelp: explain the file's purpose, domain meaning, or important assumptions\n".to_vec()
    }

    fn reset() {
        trace(|t| *t = Trace::default());
    }

    #[test]
    fn warning_record_matches_independent_bytes_and_location() {
        let mut output = Vec::new();
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut output),
            Outcome::ReportedWarning
        );
        assert_eq!(output, expected());
    }

    #[test]
    fn denial_is_policy_error_and_never_a_verification_diagnostic() {
        let mut output = Vec::new();
        let settings = Settings {
            severity: Severity::Deny,
            ..Settings::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", settings, &mut output),
            Outcome::ReportedDenial
        );
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("error[commentless-source]: documentation policy denies"));
        assert!(!text.contains("verification stopped"));
    }

    #[test]
    fn exact_sufficient_record_ceiling_succeeds() {
        let mut output = Vec::new();
        let settings = Settings {
            report_bytes: expected().len(),
            ..Settings::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", settings, &mut output),
            Outcome::ReportedWarning
        );
        assert_eq!(output, expected());
    }

    #[test]
    fn one_byte_insufficient_ceiling_fails_before_allocation() {
        reset();
        let mut output = Vec::new();
        let settings = Settings {
            report_bytes: expected().len() - 1,
            ..Settings::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", settings, &mut output),
            Outcome::ReportingLimit
        );
        assert_eq!(output, LIMIT_EMERGENCY);
        CONTROL.with(|t| assert_eq!(t.borrow().allocations, 0));
    }

    #[test]
    fn increasing_ceiling_never_increases_the_requested_payload() {
        for ceiling in [expected().len(), DEFAULT_REPORT_BYTES, MAX_REPORT_BYTES] {
            reset();
            let mut output = Vec::new();
            assert_eq!(
                report(
                    &eligible(),
                    "charge.bow",
                    Settings {
                        report_bytes: ceiling,
                        ..Settings::default()
                    },
                    &mut output,
                ),
                Outcome::ReportedWarning
            );
            CONTROL.with(|t| {
                let t = t.borrow();
                assert_eq!(t.allocations, 1);
                assert_eq!(t.request, expected().len());
                assert_eq!(t.render_calls, 1);
            });
            assert_eq!(output, expected());
        }
    }

    #[test]
    fn allocator_accepting_only_exact_demand_succeeds_under_large_ceiling() {
        reset();
        trace(|t| t.maximum_request = Some(expected().len()));
        let mut output = Vec::new();
        assert_eq!(
            report(
                &eligible(),
                "charge.bow",
                Settings {
                    report_bytes: MAX_REPORT_BYTES,
                    ..Settings::default()
                },
                &mut output,
            ),
            Outcome::ReportedWarning
        );
        CONTROL.with(|t| {
            let t = t.borrow();
            assert_eq!(t.request, expected().len());
            assert_eq!(t.allocations, 1);
        });
        assert_eq!(output, expected());
        reset();
    }

    #[test]
    fn overlong_path_sizing_stops_before_remaining_path_and_allocation() {
        reset();
        let path = "x".repeat(10_000);
        let mut output = Vec::new();
        assert_eq!(
            report(
                &eligible(),
                &path,
                Settings {
                    report_bytes: 100,
                    ..Settings::default()
                },
                &mut output,
            ),
            Outcome::ReportingLimit
        );
        CONTROL.with(|t| {
            let t = t.borrow();
            assert!(t.path_bytes <= 101);
            assert_eq!(t.allocations, 0);
        });
    }

    #[test]
    fn path_escaping_and_utf8_are_exact_and_accounted() {
        let path = "a\\\n\r\t\u{1}\u{7f}é.bow";
        let expected = b"warning[commentless-source]: this source file contains no comments\n  --> a\\\\\\n\\r\\t\\x01\\x7F\xc3\xa9.bow:2:3\nhelp: explain the file's purpose, domain meaning, or important assumptions\n";
        let mut output = Vec::new();
        assert_eq!(
            report(
                &eligible(),
                path,
                Settings {
                    report_bytes: expected.len(),
                    ..Settings::default()
                },
                &mut output,
            ),
            Outcome::ReportedWarning
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn decimal_expansion_handles_usize_boundary_without_format_allocation() {
        let mut work = Work::default();
        let mut sizing = Sizing {
            length: 0,
            ceiling: MAX_REPORT_BYTES,
        };
        number(&mut sizing, &mut work, usize::MAX).unwrap();
        assert_eq!(sizing.length, usize::MAX.to_string().len());
        assert_eq!(sizing.length, DECIMAL_DIGITS);
        let mut buffer = Record {
            bytes: Vec::with_capacity(sizing.length),
            demand: sizing.length,
        };
        number(&mut buffer, &mut work, usize::MAX).unwrap();
        assert_eq!(buffer.bytes, usize::MAX.to_string().as_bytes());
    }

    #[test]
    fn real_record_allocation_failure_is_host_reporting_failure() {
        with_fault(Fault::Allocation, || {
            let mut output = Vec::new();
            assert_eq!(
                report(&eligible(), "charge.bow", Settings::default(), &mut output),
                Outcome::HostReportingFailure
            );
            assert_eq!(output, HOST_EMERGENCY);
            CONTROL.with(|t| {
                let t = t.borrow();
                assert_eq!(t.request, expected().len());
                assert_eq!(t.allocations, 1);
                assert_eq!(t.render_calls, 0);
            });
        });
    }

    #[test]
    fn rendering_failure_is_not_successful_warning_delivery() {
        with_fault(Fault::Rendering, || {
            let mut output = Vec::new();
            assert_eq!(
                report(&eligible(), "charge.bow", Settings::default(), &mut output),
                Outcome::HostReportingFailure
            );
            assert_eq!(output, HOST_EMERGENCY);
        });
    }

    #[test]
    fn sizing_rendering_mismatch_is_rejected_before_normal_output() {
        with_fault(Fault::Mismatch, || {
            let mut output = Vec::new();
            assert_eq!(
                report(&eligible(), "charge.bow", Settings::default(), &mut output),
                Outcome::HostReportingFailure
            );
            assert_eq!(output, HOST_EMERGENCY);
        });
    }

    struct Writer {
        bytes: Vec<u8>,
        chunk: usize,
        fail_at: usize,
        interrupted: bool,
        zero: bool,
        fail_flush: bool,
        calls: usize,
        flushes: usize,
    }

    impl Default for Writer {
        fn default() -> Self {
            Self {
                bytes: Vec::new(),
                chunk: usize::MAX,
                fail_at: usize::MAX,
                interrupted: false,
                zero: false,
                fail_flush: false,
                calls: 0,
                flushes: 0,
            }
        }
    }

    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.bytes.len() >= self.fail_at {
                return Err(io::Error::new(
                    if self.interrupted {
                        io::ErrorKind::Interrupted
                    } else {
                        io::ErrorKind::BrokenPipe
                    },
                    "injected writer failure",
                ));
            }
            if self.zero {
                return Ok(0);
            }
            let count = bytes
                .len()
                .min(self.chunk)
                .min(self.fail_at - self.bytes.len());
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.fail_flush {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "flush failure"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn positive_short_writes_complete_all_bytes_and_one_flush() {
        let mut writer = Writer {
            chunk: 1,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::ReportedWarning
        );
        assert_eq!(writer.bytes, expected());
        assert_eq!(writer.calls, expected().len());
        assert_eq!(writer.flushes, 1);
    }

    #[test]
    fn zero_write_fails_normal_and_single_emergency_without_retry() {
        let mut writer = Writer {
            zero: true,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::HostReportingFailure
        );
        assert_eq!(writer.calls, 2);
        assert_eq!(writer.flushes, 0);
        assert!(writer.bytes.is_empty());
    }

    #[test]
    fn interrupted_error_is_not_retried_indefinitely() {
        let mut writer = Writer {
            fail_at: 0,
            interrupted: true,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::HostReportingFailure
        );
        assert_eq!(writer.calls, 2);
        assert_eq!(writer.flushes, 0);
    }

    #[test]
    fn error_after_partial_prefix_never_completes_a_warning() {
        let mut writer = Writer {
            fail_at: 12,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::HostReportingFailure
        );
        assert_eq!(writer.bytes, expected()[..12]);
        assert_eq!(writer.calls, 3);
        assert_eq!(writer.flushes, 0);
    }

    #[test]
    fn failure_at_final_lf_is_not_complete_delivery() {
        let mut writer = Writer {
            fail_at: expected().len() - 1,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::HostReportingFailure
        );
        assert_eq!(writer.bytes, expected()[..expected().len() - 1]);
        assert_eq!(writer.flushes, 0);
    }

    #[test]
    fn flush_failure_after_full_record_still_blocks_continuation() {
        let mut writer = Writer {
            fail_flush: true,
            ..Writer::default()
        };
        assert_eq!(
            report(&eligible(), "charge.bow", Settings::default(), &mut writer),
            Outcome::HostReportingFailure
        );
        let mut expected = expected();
        expected.extend_from_slice(HOST_EMERGENCY);
        assert_eq!(writer.bytes, expected);
        assert_eq!(writer.flushes, 2);
    }

    #[test]
    fn emergency_success_cannot_turn_limit_failure_into_warning() {
        let mut writer = Writer::default();
        assert_eq!(
            report(
                &eligible(),
                "charge.bow",
                Settings {
                    report_bytes: 1,
                    ..Settings::default()
                },
                &mut writer,
            ),
            Outcome::ReportingLimit
        );
        assert_eq!(writer.bytes, LIMIT_EMERGENCY);
        assert_eq!(writer.calls, 1);
        assert_eq!(writer.flushes, 1);
    }

    #[test]
    fn emergency_writer_failure_is_a_distinct_host_reporting_failure() {
        let mut writer = Writer {
            fail_at: 0,
            ..Writer::default()
        };
        assert_eq!(
            report(
                &eligible(),
                "charge.bow",
                Settings {
                    report_bytes: 1,
                    ..Settings::default()
                },
                &mut writer,
            ),
            Outcome::HostReportingFailure
        );
        assert_eq!(writer.calls, 1);
    }

    #[test]
    fn allow_leaves_every_reporter_fault_and_writer_unused() {
        for injected in [
            Fault::Sizing,
            Fault::Allocation,
            Fault::Rendering,
            Fault::Mismatch,
        ] {
            with_fault(injected, || {
                let mut writer = Writer {
                    fail_at: 0,
                    ..Writer::default()
                };
                assert_eq!(
                    report(
                        &eligible(),
                        "charge.bow",
                        Settings {
                            severity: Severity::Allow,
                            ..Settings::default()
                        },
                        &mut writer,
                    ),
                    Outcome::Suppressed
                );
                assert_eq!(writer.calls, 0);
                assert_eq!(writer.flushes, 0);
                CONTROL.with(|t| assert_eq!(t.borrow().sizing_calls, 0));
            });
        }
    }

    #[test]
    fn empty_and_commented_units_leave_reporter_and_writer_unused() {
        for source in [
            "",
            " \t\r\n",
            "// only",
            "// header\ntype Percent = range 0..100;",
        ] {
            for injected in [
                Fault::Sizing,
                Fault::Allocation,
                Fault::Rendering,
                Fault::Mismatch,
            ] {
                with_fault(injected, || {
                    let mut writer = Writer {
                        fail_at: 0,
                        ..Writer::default()
                    };
                    assert_eq!(
                        report(
                            &facts(source),
                            "charge.bow",
                            Settings::default(),
                            &mut writer
                        ),
                        Outcome::NoFinding
                    );
                    assert_eq!(writer.calls, 0);
                    assert_eq!(writer.flushes, 0);
                    CONTROL.with(|t| assert_eq!(t.borrow().sizing_calls, 0));
                });
            }
        }
    }

    #[test]
    fn denial_of_commented_source_has_no_finding() {
        let mut writer = Writer::default();
        assert_eq!(
            report(
                &facts("type Percent = range 0..100; // end"),
                "charge.bow",
                Settings {
                    severity: Severity::Deny,
                    ..Settings::default()
                },
                &mut writer,
            ),
            Outcome::NoFinding
        );
        assert_eq!(writer.calls, 0);
    }

    #[test]
    fn structured_exit_statuses_retain_host_and_limit_distinction() {
        for outcome in [
            Outcome::NoFinding,
            Outcome::Suppressed,
            Outcome::ReportedWarning,
        ] {
            assert_eq!(outcome.exit_code(), None);
        }
        assert_eq!(Outcome::ReportedDenial.exit_code(), Some(1));
        assert_eq!(Outcome::ReportingLimit.exit_code(), Some(1));
        assert_eq!(Outcome::HostReportingFailure.exit_code(), Some(2));
    }

    #[test]
    fn fixed_layout_and_emergency_reserves_are_explicit() {
        println!(
            "BED-037 fixed lint ledger: F={} Bdefault={} Bmax={} E={}",
            fixed_layout_bytes(),
            DEFAULT_REPORT_BYTES,
            MAX_REPORT_BYTES,
            EMERGENCY_BYTES
        );
        assert_eq!(LIMIT_EMERGENCY.len(), 70);
        assert_eq!(HOST_EMERGENCY.len(), 67);
        assert_eq!(EMERGENCY_BYTES, 70);
        assert!(fixed_layout_bytes() >= std::mem::size_of::<SourceFacts>() + DECIMAL_DIGITS);
        assert_eq!(FRAME_BYTES, DENIAL.len() + HELP.len() + 2);
        assert_eq!(Settings::default().report_bytes, 65_536);
        assert_eq!(MAX_REPORT_BYTES, 1_048_576);
    }
}
