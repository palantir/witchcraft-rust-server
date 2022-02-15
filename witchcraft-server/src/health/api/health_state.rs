use conjure_object::serde::{de, ser};
use std::fmt;
use std::str;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthState {
    #[doc = "The service node is fully operational with no issues."]
    Healthy,
    #[doc = "The service node is fully operational with no issues; however, it is requesting to defer shutdown or restart. A deferring node should not accept \"new\" jobs but should allow polling of existing jobs."]
    Deferring,
    #[doc = "The service node is no longer serving requests and is ready to be shut down. Nodes in a deferring state are expected to change to a suspended state once they have completed any pending work. A suspended node must also indicate in its readiness probe that it should not receive incoming requests."]
    Suspended,
    #[doc = "The service node is operating in a degraded state, but is capable of automatically recovering. If any of the nodes in the service were to be restarted, it may result in correctness or consistency issues with the service. Ex: When a cassandra node decides it is not up-to-date and needs to repair, the node is operating in a degraded state. Restarting the node prior to the repair being complete might result in the service being unable to correctly respond to requests."]
    Repairing,
    #[doc = "The service node is in a state that is trending towards an error. If no corrective action is taken, the health is expected to become an error."]
    Warning,
    #[doc = "The service node is operationally unhealthy."]
    Error,
    #[doc = "The service node has entered an unrecoverable state. All nodes of the service should be stopped and no automated attempt to restart the node should be made. Ex: a service fails to migrate to a new schema and is left in an unrecoverable state."]
    Terminal,
}
impl HealthState {
    #[doc = r" Returns the string representation of the enum."]
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
    fn from_plain(v: &str) -> Result<HealthState, conjure_object::plain::ParseEnumError> {
        v.parse()
    }
}
impl ser::Serialize for HealthState {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.serialize_str(self.as_str())
    }
}
impl<'de> de::Deserialize<'de> for HealthState {
    fn deserialize<D>(d: D) -> Result<HealthState, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = HealthState;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("a string")
    }
    fn visit_str<E>(self, v: &str) -> Result<HealthState, E>
    where
        E: de::Error,
    {
        match v.parse() {
            Ok(e) => Ok(e),
            Err(_) => Err(de::Error::unknown_variant(
                v,
                &[
                    "HEALTHY",
                    "DEFERRING",
                    "SUSPENDED",
                    "REPAIRING",
                    "WARNING",
                    "ERROR",
                    "TERMINAL",
                ],
            )),
        }
    }
}
