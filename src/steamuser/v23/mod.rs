use crate::{EBeginAuthSessionResult, HAuthTicket, SteamNetworkingIdentity, interface::Interface};

struct ISteamUser {

}

impl Interface for ISteamUser {
	const VERSION: &'static str =  "SteamUser023";
}

impl ISteamUser {
	pub fn get_auth_session_ticket(&self, identity: &SteamNetworkingIdentity) -> HAuthTicket {

	}

	pub fn begin_auth_session(&self, ticket: &HAuthTicket) -> EBeginAuthSessionResult {
		EBeginAuthSessionResult::InvalidTicket
	}

	pub fn end_auth_session(&self, ticket: &HAuthTicket) {

	}

	pub fn cancel_auth_ticket(&self, ticket: &HAuthTicket) {
		
	}
}