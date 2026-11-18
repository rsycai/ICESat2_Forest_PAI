#[macro_export]
macro_rules! register_source {
    ($provider:ty) => {
        $crate::__inventory::submit! { $crate::services::registry::SourceRegistration {
            id: <$provider as $crate::SourceProvider>::ID,
            provider: <$provider as $crate::SourceProvider>::PROVIDER,
            validate: <$provider as $crate::SourceProvider>::validate,
            create: <$provider as $crate::SourceProvider>::create,
        }}
    };
}

#[macro_export]
macro_rules! register_authentication {
    ($provider:ty) => {
        $crate::__inventory::submit! { $crate::services::registry::AuthenticationRegistration {
            id: <$provider as $crate::AuthenticationProvider>::ID,
            create: <$provider as $crate::AuthenticationProvider>::create,
        }}
    };
}
