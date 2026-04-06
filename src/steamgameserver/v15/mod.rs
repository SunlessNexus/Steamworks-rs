use crate::{CSteamID, interface::{Interface, ifunc}};

pub struct ISteamGameServer(*mut std::ffi::c_void);

/*
void LogOnAnonymous()
*/
ifunc!(ISteamGameServer__LogOnAnonymous, 6, 6, std::ffi::c_void, ());

/*
CSteamID GetSteamID()
*/
ifunc!(ISteamGameServer__GetSteamID, 10, 10, u64, ());

impl Interface for ISteamGameServer {
	const VERSION: &'static str = "SteamGameServer015\0";

	fn object_ptr(&self) -> *mut std::ffi::c_void {
		self.0
	}

	fn create(object_ptr: *mut std::ffi::c_void) -> Self {
		ISteamGameServer(object_ptr)
	}
}

impl ISteamGameServer {
	pub fn log_on_anonymous(&self) {
		ISteamGameServer__LogOnAnonymous(
			self
		);
	}

	pub fn get_steam_id(&self) -> CSteamID {
		ISteamGameServer__GetSteamID(
			self
		).into()
	}
}