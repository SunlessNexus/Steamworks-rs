#[allow(unused)]
pub mod steamuser;
pub mod interface;

#[allow(dead_code, unused)]
mod dl;

#[derive(Debug)]
pub enum InitError {
	NotAFile,
	Missing,
	FailedLoading,
	FailedInit,
	Internal
}

pub struct Context {
	#[allow(unused)]
	library: dl::Handle,
	c_fn_shutdown: unsafe extern "C" fn() -> std::ffi::c_void
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			(self.c_fn_shutdown)();
		}
	}
}

pub fn init(path: std::path::PathBuf, interfaces: Vec<&'static str>) -> Result<Context, InitError> {
	if !path.is_file() {
		return Err(InitError::NotAFile);
	}

	if !path.exists() {
		return Err(InitError::Missing);
	}

	let path = match path.into_os_string().into_string() {
		Ok(path) => path,
		Err(_) => {
			println!("Impossible to handle this path. Contact steamworks crate authors.");
			eprintln!("Impossible to handle this path. Contact steamworks crate authors.");
			return Err(InitError::Internal)
		}
	};

	let module = match dl::open(path) {
		Some(module) => module,
		None => return Err(InitError::FailedLoading)
	};

	let init = module.find(b"SteamInternal_SteamAPI_Init\0");
	if init.is_null() {
		return Err(InitError::FailedInit)
	}

	let shutdown = module.find(b"SteamAPI_Shutdown\0");
	if shutdown.is_null() {
		return Err(InitError::FailedInit)
	}

	let init: unsafe extern "C" fn(*const u8, *mut u8) -> u8 = unsafe { std::mem::transmute(init) };
	unsafe {
		let mut version: Vec<u8> = Vec::with_capacity(1024);
		for interface in interfaces {
			version.extend_from_slice(interface.as_bytes());
			version.push(b'\0');
		}
		version.push(b'\0');

		let mut message: [u8; 1024] = [0; 1024];
		if init(version.as_ptr(), message.as_mut_ptr()) != 0x0 {
			message[1023] = b'\0';
			println!("[Steamworks-rs] Failed to init SteamAPI: {}", String::from_utf8_lossy(&message));
			return Err(InitError::FailedInit)
		}
	}

	Ok(Context {
		library: module,
		c_fn_shutdown: unsafe { std::mem::transmute(shutdown) }
	})
}

pub struct CSteamID(u64);

// Opaque object representing an authticket
pub struct HAuthTicket {
	pub(crate) version: u32,
	// Version 1
	pub(crate) v1_handle: Option<u32>,
	pub(crate) v1_associated_id: Option<CSteamID>,
	pub(crate) v1_buffer: Option<[u8; 1024]>,
	pub(crate) v1_length: Option<usize>
}

pub struct SteamNetworkingIdentity {

}

pub enum EBeginAuthSessionResult {
	OK,						// Ticket is valid for this game and this steamID.
	InvalidTicket,			// Ticket is not valid.
	DuplicateRequest,		// A ticket has already been submitted for this steamID
	InvalidVersion,			// Ticket is from an incompatible interface version
	GameMismatch,			// Ticket is not for this game
	ExpiredTicket
}

pub enum Universe {
	Individual,
	Public,
	Beta,
	Internal,
	Dev,
	RC
}

pub enum AccountType {
	Invalid,
	Individual,
	Multiseat,
	GameServer,
	AnonGameServer,
	Pending,
	ContentServer,
	Clan,
	Chat,
	P2P,
	AnonUser
}

impl From<u64> for CSteamID {
	fn from(id: u64) -> Self {
		Self(id)
	}
}

impl Into<u64> for CSteamID {
	fn into(self) -> u64 {
		self.0
	}
}

impl Into<u64> for &CSteamID {
	fn into(self) -> u64 {
		self.0
	}
}

impl CSteamID {
	pub fn new(universe: Universe, account_type: AccountType, instance: u32, account_id: u32) -> Self {
		let universe: u8 = match universe {
			Universe::Individual => 0,
			Universe::Public => 1,
			Universe::Beta => 2,
			Universe::Internal => 3,
			Universe::Dev => 4,
			Universe::RC => 5
		};

		let account_type: u8 = match account_type {
			AccountType::Invalid => 0,
			AccountType::Individual => 1,
			AccountType::Multiseat => 2,
			AccountType::GameServer => 3,
			AccountType::AnonGameServer => 4,
			AccountType::Pending => 5,
			AccountType::ContentServer => 6,
			AccountType::Clan => 7,
			AccountType::Chat => 8,
			AccountType::P2P => 9,
			AccountType::AnonUser => 10
		};

		let instance = instance & 0xFFFFF;

		// Shift every bits in their respective place
		Self( ((universe as u64) << 56) | ((account_type as u64) << 52) | ((instance as u64) << 32) | (account_id as u64) )
	}

	pub fn universe(&self) -> Option<Universe> {
		match ((self.0 >> 56) & 0xFF) as u8 {
			0 => Some(Universe::Individual),
			1 => Some(Universe::Public),
			2 => Some(Universe::Beta),
			3 => Some(Universe::Internal),
			4 => Some(Universe::Dev),
			5 => Some(Universe::RC),
			_ => None
		}
	}

	pub fn account_type(&self) -> Option<AccountType> {
		match ((self.0 >> 52) & 0xF) as u8 {
			0 => Some(AccountType::Invalid),
			1 => Some(AccountType::Individual),
			2 => Some(AccountType::Multiseat),
			3 => Some(AccountType::GameServer),
			4 => Some(AccountType::AnonGameServer),
			5 => Some(AccountType::Pending),
			6 => Some(AccountType::ContentServer),
			7 => Some(AccountType::Clan),
			8 => Some(AccountType::Chat),
			9 => Some(AccountType::P2P),
			10 => Some(AccountType::AnonUser),
			_ => None
		}
	}

	pub fn instance(&self) -> u32 {
		((self.0 >> 32) & 0xFFFFF) as u32
	}

	pub fn account_id(&self) -> u32 {
		(self.0 & 0xFFFFFFFF) as u32
	}
}

impl HAuthTicket {
	pub(crate) fn new_version1(handle: u32, steam_id: CSteamID, buffer: [u8; 1024], length: usize) -> Self {
		Self {
			version: 1,
			v1_handle: Some(handle),
			v1_associated_id: Some(steam_id),
			v1_buffer: Some(buffer),
			v1_length: Some(length)
		}
	}
}

impl EBeginAuthSessionResult {
	pub(crate) fn v1_from(value: u32) -> Option<Self> {
		match value {
			0 => Some(EBeginAuthSessionResult::OK),
			1 => Some(EBeginAuthSessionResult::InvalidTicket),
			2 => Some(EBeginAuthSessionResult::DuplicateRequest),
			3 => Some(EBeginAuthSessionResult::InvalidVersion),
			4 => Some(EBeginAuthSessionResult::GameMismatch),
			5 => Some(EBeginAuthSessionResult::ExpiredTicket),
			_ => None
		}
	}
}