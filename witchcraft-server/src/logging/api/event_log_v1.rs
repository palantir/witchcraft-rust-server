///Definition of the event.1 format.
#[derive(
    Debug,
    Clone,
    conjure_object::serde::Serialize,
    conjure_object::serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash
)]
#[serde(crate = "conjure_object::serde")]
#[conjure_object::private::staged_builder::staged_builder]
#[builder(crate = conjure_object::private::staged_builder, update, inline)]
pub struct EventLogV1 {
    #[builder(into)]
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "time")]
    time: conjure_object::DateTime<conjure_object::Utc>,
    #[builder(into)]
    #[serde(rename = "eventName")]
    event_name: String,
    #[builder(into)]
    #[serde(rename = "eventType")]
    event_type: String,
    #[builder(
        default,
        map(
            key(type = String, into),
            value(
                custom(
                    type = impl
                    conjure_object::serde::Serialize,
                    convert = |v|conjure_object::Any::new(
                        v
                    ).expect("value failed to serialize")
                )
            )
        )
    )]
    #[serde(
        rename = "values",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    values: std::collections::BTreeMap<String, conjure_object::Any>,
    #[builder(default, into)]
    #[serde(rename = "uid", skip_serializing_if = "Option::is_none", default)]
    uid: Option<super::UserId>,
    #[builder(default, into)]
    #[serde(rename = "sid", skip_serializing_if = "Option::is_none", default)]
    sid: Option<super::SessionId>,
    #[builder(default, into)]
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none", default)]
    token_id: Option<super::TokenId>,
    #[builder(default, into)]
    #[serde(rename = "orgId", skip_serializing_if = "Option::is_none", default)]
    org_id: Option<super::OrganizationId>,
    #[builder(
        default,
        map(
            key(type = String, into),
            value(
                custom(
                    type = impl
                    conjure_object::serde::Serialize,
                    convert = |v|conjure_object::Any::new(
                        v
                    ).expect("value failed to serialize")
                )
            )
        )
    )]
    #[serde(
        rename = "unsafeParams",
        skip_serializing_if = "std::collections::BTreeMap::is_empty",
        default
    )]
    unsafe_params: std::collections::BTreeMap<String, conjure_object::Any>,
}
impl EventLogV1 {
    #[inline]
    pub fn type_(&self) -> &str {
        &*self.type_
    }
    #[inline]
    pub fn time(&self) -> conjure_object::DateTime<conjure_object::Utc> {
        self.time
    }
    ///Dot-delimited name of event, e.g. `com.foundry.compass.api.Compass.http.ping.failures`
    #[inline]
    pub fn event_name(&self) -> &str {
        &*self.event_name
    }
    ///Type of event being represented, e.g. `gauge`, `histogram`, `counter`
    #[inline]
    pub fn event_type(&self) -> &str {
        &*self.event_type
    }
    ///Observations, measurements and context associated with the event
    #[inline]
    pub fn values(&self) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.values
    }
    ///User id (if available)
    #[inline]
    pub fn uid(&self) -> Option<&super::UserId> {
        self.uid.as_ref().map(|o| &*o)
    }
    ///Session id (if available)
    #[inline]
    pub fn sid(&self) -> Option<&super::SessionId> {
        self.sid.as_ref().map(|o| &*o)
    }
    ///API token id (if available)
    #[inline]
    pub fn token_id(&self) -> Option<&super::TokenId> {
        self.token_id.as_ref().map(|o| &*o)
    }
    ///Organization id (if available)
    #[inline]
    pub fn org_id(&self) -> Option<&super::OrganizationId> {
        self.org_id.as_ref().map(|o| &*o)
    }
    ///Unsafe metadata describing the event
    #[inline]
    pub fn unsafe_params(
        &self,
    ) -> &std::collections::BTreeMap<String, conjure_object::Any> {
        &self.unsafe_params
    }
}
