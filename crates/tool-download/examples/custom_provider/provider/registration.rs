use crate::provider::source::JsonSource;
use tool_download::Result;
use tool_download::SourceAdapter;
use tool_download::SourceContext;
use tool_download::SourceProvider;

struct JsonProvider;
impl SourceProvider for JsonProvider {
    const ID: &'static str = "json-example";
    const PROVIDER: &'static str = "custom";
    fn create(context: SourceContext<'_>) -> Result<SourceAdapter> {
        let (product, release) = context
            .definition
            .selection(context.product, context.release)?;
        Ok(SourceAdapter::single(
            context.definition,
            JsonSource {
                id: context.definition.id.clone(),
                endpoint: context.definition.search_url.clone(),
                product: product.into(),
                release: release.into(),
            },
            tool_download::services::storage::StandardStorage::default(),
        ))
    }
}
tool_download::register_source!(JsonProvider);
