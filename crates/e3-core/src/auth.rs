use crate::client::MoodleClient;
use crate::error::Result;
use crate::types::SiteInfo;

/// Get site info (validates token/session)
pub async fn get_site_info(client: &MoodleClient) -> Result<SiteInfo> {
    client
        .call("core_webservice_get_site_info", &serde_json::json!({}))
        .await
}
