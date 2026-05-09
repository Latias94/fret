use serde::{Deserialize, Serialize};

use crate::{MechanismCase, MechanismSuite, ObservedTree, PredicateFailure, evaluate_predicate};

pub trait ScenarioObserver<S> {
    fn observe_case(
        &mut self,
        case: &MechanismCase<S>,
    ) -> Result<ObservedTree, ScenarioObserveError>;
}

impl<S, F> ScenarioObserver<S> for F
where
    F: FnMut(&MechanismCase<S>) -> Result<ObservedTree, ScenarioObserveError>,
{
    fn observe_case(
        &mut self,
        case: &MechanismCase<S>,
    ) -> Result<ObservedTree, ScenarioObserveError> {
        self(case)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ScenarioObserveError {
    pub message: String,
}

impl ScenarioObserveError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MechanismHarness;

impl MechanismHarness {
    pub fn new() -> Self {
        Self
    }

    pub fn run_suite<S, O>(&self, suite: &MechanismSuite<S>, observer: &mut O) -> MechanismReport
    where
        O: ScenarioObserver<S>,
    {
        let mut report = MechanismReport {
            suite_id: suite.suite_id.clone(),
            cases: Vec::with_capacity(suite.cases.len()),
        };

        for case in &suite.cases {
            report.cases.push(self.run_case(case, observer));
        }

        report
    }

    pub fn run_case<S, O>(&self, case: &MechanismCase<S>, observer: &mut O) -> CaseReport
    where
        O: ScenarioObserver<S>,
    {
        let mut failures = Vec::new();
        let observed = match observer.observe_case(case) {
            Ok(observed) => observed,
            Err(err) => {
                failures.push(format!("observe failed: {}", err.message));
                return CaseReport {
                    case_id: case.id.clone(),
                    passed: false,
                    failures,
                };
            }
        };

        for (idx, predicate) in case.oracle.predicates.iter().enumerate() {
            if let Err(PredicateFailure { message }) = evaluate_predicate(&observed, predicate) {
                failures.push(format!("predicate[{idx}] failed: {message}"));
            }
        }

        CaseReport {
            case_id: case.id.clone(),
            passed: failures.is_empty(),
            failures,
        }
    }

    pub fn assert_suite_passes<S, O>(&self, suite: &MechanismSuite<S>, observer: &mut O)
    where
        O: ScenarioObserver<S>,
    {
        let report = self.run_suite(suite, observer);
        assert!(
            report.passed(),
            "mechanism suite failed:\n{}",
            report.failure_summary()
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismReport {
    pub suite_id: String,
    pub cases: Vec<CaseReport>,
}

impl MechanismReport {
    pub fn passed(&self) -> bool {
        self.cases.iter().all(|case| case.passed)
    }

    pub fn failure_summary(&self) -> String {
        let mut out = Vec::new();
        out.push(format!("suite={}", self.suite_id));
        for case in self.cases.iter().filter(|case| !case.passed) {
            out.push(format!("case={}", case.case_id));
            for failure in &case.failures {
                out.push(format!("  - {failure}"));
            }
        }
        out.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseReport {
    pub case_id: String,
    pub passed: bool,
    pub failures: Vec<String>,
}
