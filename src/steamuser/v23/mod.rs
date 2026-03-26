use crate::{EBeginAuthSessionResult, HAuthTicket, SteamNetworkingIdentity, interface::Interface, interface::ifunc};

struct ISteamUser {
	object: *mut std::ffi::c_void
}

/*
HAuthTicket GetAuthSessionTicket(
	void *pTicket,
	int cbMaxTicket,
	uint32 *pcbTicket,
	const SteamNetworkingIdentity *pSteamNetworkingIdentity
)
*/
ifunc!(ISteamUser__GetAuthSessionTicket, 0, 0, std::ffi::c_uint, (
	pTicket: *mut std::ffi::c_void,
	cbMaxTicket: std::ffi::c_int,
	pcbTicket: *mut std::ffi::c_uint,
	pSteamNetworkingIdentity: *const std::ffi::c_void)
);

impl Interface for ISteamUser {
	const VERSION: &'static str =  "SteamUser023";

	fn object_ptr(&self) -> *mut std::ffi::c_void {
		self.object
	}
}

impl ISteamUser {
	pub fn get_auth_session_ticket(&self, identity: &SteamNetworkingIdentity) -> HAuthTicket {
		let mut ticket: [u8; 1024] = [0; 1024];
		let mut length: u32 = 0;
		
		ISteamUser__GetAuthSessionTicket(
			self,
			ticket.as_mut_ptr() as *mut std::ffi::c_void,
			ticket.len() as i32,
			&mut length as *mut u32,
			std::ptr::null::<std::ffi::c_void>()
		);
		todo!()
	}

	pub fn begin_auth_session(&self, ticket: &HAuthTicket) -> EBeginAuthSessionResult {
		EBeginAuthSessionResult::InvalidTicket
	}

	pub fn end_auth_session(&self, ticket: &HAuthTicket) {

	}

	pub fn cancel_auth_ticket(&self, ticket: &HAuthTicket) {
		
	}
}