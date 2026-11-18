use crate::config::catalog::{AuthenticationProfile, SourceDefinition};
use crate::{Authentication, Result, SourceAdapter, SourceContext};

pub struct SourceRegistration {
    pub id: &'static str,
    pub provider: &'static str,
    pub validate: fn(&SourceDefinition) -> Result<()>,
    pub create: for<'a> fn(SourceContext<'a>) -> Result<SourceAdapter>,
}
inventory::collect!(SourceRegistration);

pub struct AuthenticationRegistration {
    pub id: &'static str,
    pub create: fn(&AuthenticationProfile) -> Result<Box<dyn Authentication>>,
}
inventory::collect!(AuthenticationRegistration);
