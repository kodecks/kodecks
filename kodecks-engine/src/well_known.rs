use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnown {
    pub servers: Vec<Server>,
}

impl WellKnown {
    pub fn find_latest(&self, client_version: &Version) -> Option<Server> {
        let mut servers = self.servers.clone();
        servers.sort_by(|a, b| b.server_version.cmp(&a.server_version));
        servers.into_iter().find(|server| {
            server
                .client_version_requirement
                .as_ref()
                .map_or(true, |req| req.matches(client_version))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: Url,
    pub server_version: Option<Version>,
    pub client_version_requirement: Option<VersionReq>,
}
