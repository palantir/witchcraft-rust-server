#![allow(deprecated)]
use std::fmt;
use std::str;
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    conjure_object::serde::Deserialize,
    conjure_object::serde::Serialize,
)]
#[serde(crate = "conjure_object::serde")]
pub enum HealthState {
    ///The service node is fully operational with no issues.
    #[serde(rename = "HEALTHY")]
    Healthy,
    ///The service node is fully operational with no issues; however, it is requesting to defer shutdown or restart. A deferring node should not accept "new" jobs but should allow polling of existing jobs.
    #[serde(rename = "DEFERRING")]
    Deferring,
    ///The service node is no longer serving requests and is ready to be shut down. Nodes in a deferring state are expected to change to a suspended state once they have completed any pending work. A suspended node must also indicate in its readiness probe that it should not receive incoming requests.
    #[serde(rename = "SUSPENDED")]
    Suspended,
    ///The service node is operating in a degraded state, but is capable of automatically recovering. If any of the nodes in the service were to be restarted, it may result in correctness or consistency issues with the service. Ex: When a cassandra node decides it is not up-to-date and needs to repair, the node is operating in a degraded state. Restarting the node prior to the repair being complete might result in the service being unable to correctly respond to requests.
    #[serde(rename = "REPAIRING")]
    Repairing,
    ///The service node is in a state that is trending towards an error. If no corrective action is taken, the health is expected to become an error.
    #[serde(rename = "WARNING")]
    Warning,
    ///The service node is operationally unhealthy.
    #[serde(rename = "ERROR")]
    Error,
    ///The service node has entered an unrecoverable state. All nodes of the service should be stopped and no automated attempt to restart the node should be made. Ex: a service fails to migrate to a new schema and is left in an unrecoverable state.
    #[serde(rename = "TERMINAL")]
    Terminal,
}
impl HealthState {
    /// Returns the string representation of the enum.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            HealthState::Healthy => "HEALTHY",
            HealthState::Deferring => "DEFERRING",
            HealthState::Suspended => "SUSPENDED",
            HealthState::Repairing => "REPAIRING",
            HealthState::Warning => "WARNING",
            HealthState::Error => "ERROR",
            HealthState::Terminal => "TERMINAL",
        }
    }
}
impl fmt::Display for HealthState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}
impl conjure_object::Plain for HealthState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        conjure_object::Plain::fmt(self.as_str(), fmt)
    }
}
impl str::FromStr for HealthState {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_str(v: &str) -> Result<HealthState, conjure_object::plain::ParseEnumError> {
        match v {
            "HEALTHY" => Ok(HealthState::Healthy),
            "DEFERRING" => Ok(HealthState::Deferring),
            "SUSPENDED" => Ok(HealthState::Suspended),
            "REPAIRING" => Ok(HealthState::Repairing),
            "WARNING" => Ok(HealthState::Warning),
            "ERROR" => Ok(HealthState::Error),
            "TERMINAL" => Ok(HealthState::Terminal),
            _ => Err(conjure_object::plain::ParseEnumError::new()),
        }
    }
}
impl conjure_object::FromPlain for HealthState {
    type Err = conjure_object::plain::ParseEnumError;
    #[inline]
    fn from_plain(
        v: &str,
    ) -> Result<HealthState, conjure_object::plain::ParseEnumError> {
        v.parse()
    }
}
