use crate::{CSteamID, EBeginAuthSessionResult, HAuthTicket, SteamNetworkingIdentity, interface::{Interface, ifunc}};

pub struct ISteamUser(crate::Context, *mut std::ffi::c_void);

/*
CSteamID GetSteamID()
*/
ifunc!(ISteamUser__GetSteamID, 2, 2, u64, ());

/*
HAuthTicket GetAuthSessionTicket(
	void *pTicket,
	int cbMaxTicket,
	uint32 *pcbTicket,
	const SteamNetworkingIdentity *pSteamNetworkingIdentity
)
*/
ifunc!(ISteamUser__GetAuthSessionTicket, 13, 13, std::ffi::c_uint, (
	pTicket: *mut std::ffi::c_void,
	cbMaxTicket: std::ffi::c_int,
	pcbTicket: *mut std::ffi::c_uint,
	pSteamNetworkingIdentity: *const std::ffi::c_void
));

/*
EBeginAuthSessionResult BeginAuthSession( const void *pAuthTicket, int cbAuthTicket, CSteamID steamID );
*/
ifunc!(ISteamUser__BeginAuthSession, 15, 15, std::ffi::c_uint, (
	pTicket: *const std::ffi::c_void,
	cbAuthTicket: std::ffi::c_uint,
	steamID: u64
));

/*
void EndAuthSession( CSteamID steamID );
*/
ifunc!(ISteamUser__EndAuthSession, 16, 16, std::ffi::c_void, (
	steamID: u64
));

/*
void CancelAuthTicket( HAuthTicket hAuthTicket );
*/
ifunc!(ISteamUser__CancelAuthTicket, 17, 17, std::ffi::c_void, (
	hAuthTicket: u32
));

/*
SteamAPICall_t RequestEncryptedAppTicket( void *pDataToInclude, int cbDataToInclude );
*/
ifunc!(ISteamUser__RequestEncryptedAppTicket, 21, 21, u64, (
	pDataToInclude: *const u8,
	cbDataToInclude: std::ffi::c_int
));

#[allow(non_snake_case)]
#[repr(C)]
struct EncryptedAppTicketResponse_t {
	result: u8
}

struct EncryptedAppTicketResponse {
	result: crate::EBeginAuthSessionResult
}

impl Interface for ISteamUser {
	const VERSION: &'static str = "SteamUser023\0";

	fn object_ptr(&self) -> *mut std::ffi::c_void {
		self.1
	}

	fn create(object_ptr: *mut std::ffi::c_void, context: crate::Context) -> Self {
		ISteamUser(context, object_ptr)
	}

	fn linked_context(&self) -> &crate::Context {
		&self.0
	}
}

impl ISteamUser {
	pub fn get_steam_id(&self) -> CSteamID {
		ISteamUser__GetSteamID(
			self
		).into()
	}

	pub fn get_auth_session_ticket(&self, identity: &SteamNetworkingIdentity) -> Option<HAuthTicket> {
		let mut ticket: [u8; 1024] = [0; 1024];
		let mut length: u32 = 0;
		
		let handle = ISteamUser__GetAuthSessionTicket(
			self,
			ticket.as_mut_ptr() as *mut std::ffi::c_void,
			ticket.len() as i32,
			&mut length as *mut u32,
			std::ptr::null::<std::ffi::c_void>()
		);
		
		if handle == 0 || length <= 0 {
			None
		} else {
			Some(
				HAuthTicket::new_version1(
					handle,
					self.get_steam_id(),
					ticket,
					length as usize
				)
			)
		}
	}

	pub fn begin_auth_session(&self, ticket: &HAuthTicket) -> EBeginAuthSessionResult {
		if ticket.version != 1 {
			// We don't know how to handle this ticket
			return EBeginAuthSessionResult::InvalidTicket
		}
		match EBeginAuthSessionResult::v1_from(ISteamUser__BeginAuthSession(
			self,
			ticket.v1_buffer.unwrap().as_ptr() as *const std::ffi::c_void,
			ticket.v1_length.as_ref().unwrap().clone() as u32,
			ticket.v1_associated_id.as_ref().unwrap().into()
		)) {
			// If this doesn't branch through here, this is a steam bug
			Some(result) => result,
			// Though rather than unwrapping on the value and crash
			// Assume the result is some form of invalid ticket
			_ => EBeginAuthSessionResult::InvalidTicket
		}
	}

	pub fn end_auth_session(&self, ticket: HAuthTicket) {
		if ticket.version != 1 {
			// We don't know how to handle this ticket
			return
		}

		ISteamUser__EndAuthSession(
			self,
			ticket.v1_associated_id.unwrap().into()
		);
	}

	pub fn cancel_auth_ticket(&self, ticket: HAuthTicket) {
		if ticket.version != 1 {
			// We don't know how to handle this ticket
			return
		}

		ISteamUser__CancelAuthTicket(
			self,
			ticket.v1_handle.unwrap()
		);
	}

	pub fn request_encrypted_app_ticket(&self, data: &[u8]) {
		let result = ISteamUser__RequestEncryptedAppTicket(
			self,
			data.as_ptr(),
			data.len() as i32
		);

		self.linked_context()
	}
}