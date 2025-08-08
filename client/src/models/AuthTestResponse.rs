use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthTestResponse {
    pub ok: bool,
    pub url: String,
    pub team: String,
    pub user: String,
    pub team_id: String,
    pub user_id: String,
    pub is_enterprise_install: bool, // not in documentation, but is in response
}
