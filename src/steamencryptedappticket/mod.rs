use crate::CSteamID;

pub struct SteamEncryptedAppTicket {
	decrypted: Vec<u8>,

	fn_is_ticket_for_app: unsafe extern "C" fn(*const u8, u32, u32) -> bool,
	fn_get_ticket_issue_time: unsafe extern "C" fn(*const u8, u32) -> u32,
	fn_get_ticket_steam_id: unsafe extern "C" fn(*const u8, u32, *mut u64) -> std::ffi::c_void,
	fn_get_ticket_app_id: unsafe extern "C" fn(*const u8, u32) -> u32,
	fn_user_owns_app_in_ticket: unsafe extern "C" fn(*const u8, u32, u32) -> bool,
	fn_user_is_vac_banned: unsafe extern "C" fn(*const u8, u32) -> bool,
	fn_get_app_defined_value: unsafe extern "C" fn(*const u8, u32, *mut u32) -> bool,
	fn_get_user_variable_data: unsafe extern "C" fn(*const u8, u32, *mut u32) -> *const u8,
	fn_is_ticket_signed: unsafe extern "C" fn(*const u8, u32, *const u8, u32) -> bool,
	fn_is_license_borrowed: unsafe extern "C" fn(*const u8, u32) -> bool,
	fn_is_license_temporary: unsafe extern "C" fn(*const u8, u32) -> bool
}

impl SteamEncryptedAppTicket {
	pub fn is_for_app(&self, app_id: u32) -> bool {
		unsafe { (self.fn_is_ticket_for_app)(self.decrypted.as_ptr(), self.decrypted.len() as u32, app_id) }
	}

	pub fn get_issue_time(&self) -> std::time::SystemTime {
		let seconds = unsafe { (self.fn_get_ticket_issue_time)(self.decrypted.as_ptr(), self.decrypted.len() as u32) };
		std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds as u64)
	}

	pub fn get_steam_id(&self) -> CSteamID {
		let mut steamid: u64 = 0;
		unsafe { (self.fn_get_ticket_steam_id)(self.decrypted.as_ptr(), self.decrypted.len() as u32, &mut steamid) };
		steamid.into()
	}

	pub fn get_app_id(&self) -> u32 {
		unsafe { (self.fn_get_ticket_app_id)(self.decrypted.as_ptr(), self.decrypted.len() as u32) }
	}

	pub fn user_owns_app_in_ticket(&self, app_id: u32) -> bool {
		unsafe { (self.fn_user_owns_app_in_ticket)(self.decrypted.as_ptr(), self.decrypted.len() as u32, app_id) }
	}

	pub fn user_is_vac_banned(&self) -> bool {
		unsafe { (self.fn_user_is_vac_banned)(self.decrypted.as_ptr(), self.decrypted.len() as u32) }
	}

	pub fn get_app_defined_value(&self) -> u32 {
		let mut ret: u32 = 0;
		unsafe { (self.fn_get_app_defined_value)(self.decrypted.as_ptr(), self.decrypted.len() as u32, &mut ret) };
		ret
	}

	pub fn is_license_borrowed(&self) -> bool {
		unsafe { (self.fn_is_license_borrowed)(self.decrypted.as_ptr(), self.decrypted.len() as u32) }
	}

	pub fn is_license_temporary(&self) -> bool {
		unsafe { (self.fn_is_license_temporary)(self.decrypted.as_ptr(), self.decrypted.len() as u32) }
	}
}