use std::sync::Arc;

#[allow(unused)]
pub mod steamuser;
#[allow(unused)]
pub mod steamgameserver;
#[allow(unused)]
pub mod steamencryptedappticket;
pub mod interface;
pub mod callback;

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

#[derive(Clone)]
pub(crate) struct HSteamPipe(i32);
#[derive(Clone)]
pub(crate) struct HSteamUser(i32);

pub(crate) struct InnerContext {
	library: dl::Handle,
	c_fn_shutdown: unsafe extern "C" fn() -> std::ffi::c_void
}

impl Drop for InnerContext {
	fn drop(&mut self) {
		unsafe {
			(self.c_fn_shutdown)();
		}
	}
}

impl InnerContext {
	pub(crate) fn new(lib: dl::Handle, fn_shutdown: unsafe extern "C" fn() -> std::ffi::c_void) -> Self {
		Self {
			library: lib,
			c_fn_shutdown: fn_shutdown,
		}
	}
}

#[derive(Clone)]
pub struct Context {
	inner_context: Arc<InnerContext>,
	c_fn_create_interface: unsafe extern "C" fn(std::ffi::c_int, *const u8) -> *mut std::ffi::c_void,
	user: HSteamUser,
	pipe: HSteamPipe
}

fn load_lib(path: std::path::PathBuf) -> Result<dl::Handle, InitError> {
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

	match dl::open(path) {
		Some(module) => Ok(module),
		None => Err(InitError::FailedLoading)
	}
}

pub fn init_gameserver(
	path: std::path::PathBuf,
	app_id: u32,
	interfaces: Vec<&'static str>,
	ip: std::net::IpAddr,
	gameport: u16,
	queryport: u16,
	servermode: EServerMode,
	version: &str
) -> Result<Context, InitError> {
	let ip = match ip {
		std::net::IpAddr::V4(ip) => ip,
		std::net::IpAddr::V6(_) => return Err(InitError::FailedInit)
	};

	let module = load_lib(path)?;

	let init = module.find(b"SteamInternal_GameServer_Init_V2\0");
	if init.is_null() {
		return Err(InitError::FailedInit)
	}

	let shutdown = module.find(b"SteamGameServer_Shutdown\0");
	if shutdown.is_null() {
		return Err(InitError::FailedInit)
	}

	let create_interface = module.find(b"SteamInternal_FindOrCreateGameServerInterface\0");
	if create_interface.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteampipe = module.find(b"SteamGameServer_GetHSteamPipe\0");
	if gethsteampipe.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteampipe: unsafe extern "C" fn() -> i32 = unsafe {
		std::mem::transmute(gethsteampipe)
	};

	let gethsteamuser = module.find(b"SteamGameServer_GetHSteamUser\0");
	if gethsteamuser.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteamuser: unsafe extern "C" fn() -> i32 = unsafe {
		std::mem::transmute(gethsteamuser)
	};

	unsafe {
		let mut versioncheck: Vec<u8> = Vec::with_capacity(1024);
		for interface in interfaces {
			versioncheck.extend_from_slice(interface.as_bytes());
			versioncheck.push(b'\0');
		}
		versioncheck.push(b'\0');

		let mut message: [u8; 1024] = [0; 1024];
		// ESteamAPIInitResult SteamInternal_GameServer_Init_V2( uint32 unIP, uint16 usGamePort, uint16 usQueryPort, EServerMode eServerMode, const char *pchVersionString, const char *pszInternalCheckInterfaceVersions, SteamErrMsg *pOutErrMsg );
		let init: unsafe extern "C" fn(u32, u16, u16, u8, *const u8, *const u8, *mut u8) -> u8 = std::mem::transmute(init);

		std::env::set_var("SteamAppId", format!("{}", app_id));
		std::env::set_var("SteamGameId", format!("{}", app_id));
		if init(ip.to_bits(), gameport, queryport, servermode.v1_into().unwrap() as u8, version.as_ptr(), versioncheck.as_ptr(), message.as_mut_ptr()) != 0x0 {
			message[1023] = b'\0';
			println!("[Steamworks-rs] Failed to init SteamAPI: {}", String::from_utf8_lossy(&message));
			return Err(InitError::FailedInit)
		}
	}

	println!("c_fn_create_interface: {:#x}", create_interface.addr());

	Ok(Context {
		inner_context: InnerContext::new(module, unsafe { std::mem::transmute(shutdown) }).into(),
		c_fn_create_interface: unsafe {
			std::mem::transmute(create_interface)
		},
		pipe: HSteamPipe( unsafe {
			gethsteampipe()
		} ),
		user: HSteamUser( unsafe {
			gethsteamuser()
		})
	})
}

pub fn init(
	path: std::path::PathBuf,
	app_id: u32,
	interfaces: Vec<&'static str>
) -> Result<Context, InitError> {
	let module = load_lib(path)?;

	let init = module.find(b"SteamInternal_SteamAPI_Init\0");
	if init.is_null() {
		return Err(InitError::FailedInit)
	}

	let shutdown = module.find(b"SteamAPI_Shutdown\0");
	if shutdown.is_null() {
		return Err(InitError::FailedInit)
	}

	let create_interface = module.find(b"SteamInternal_FindOrCreateUserInterface\0");
	if create_interface.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteampipe = module.find(b"SteamAPI_GetHSteamPipe\0");
	if gethsteampipe.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteampipe: unsafe extern "C" fn() -> i32 = unsafe {
		std::mem::transmute(gethsteampipe)
	};

	let gethsteamuser = module.find(b"SteamAPI_GetHSteamUser\0");
	if gethsteamuser.is_null() {
		return Err(InitError::FailedInit)
	}

	let gethsteamuser: unsafe extern "C" fn() -> i32 = unsafe {
		std::mem::transmute(gethsteamuser)
	};

	unsafe {
		let mut version: Vec<u8> = Vec::with_capacity(1024);
		for interface in interfaces {
			version.extend_from_slice(interface.as_bytes());
			version.push(b'\0');
		}
		version.push(b'\0');

		let mut message: [u8; 1024] = [0; 1024];
		let init: unsafe extern "C" fn(*const u8, *mut u8) -> u8 = std::mem::transmute(init);

		std::env::set_var("SteamAppId", format!("{}", app_id));
		std::env::set_var("SteamGameId", format!("{}", app_id));
		if init(version.as_ptr(), message.as_mut_ptr()) != 0x0 {
			message[1023] = b'\0';
			println!("[Steamworks-rs] Failed to init SteamAPI: {}", String::from_utf8_lossy(&message));
			return Err(InitError::FailedInit)
		}
	}

	Ok(Context {
		inner_context: InnerContext::new(module, unsafe { std::mem::transmute(shutdown) }).into(),
		c_fn_create_interface: unsafe {
			std::mem::transmute(create_interface)
		},
		pipe: HSteamPipe( unsafe {
			gethsteampipe()
		} ),
		user: HSteamUser( unsafe {
			gethsteamuser()
		})
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

pub enum EResult {
	None,                                       // no result
	Ok,                                         // success
	Fail,                                       // generic failure 
	NoConnection,                               // no/failed network connection
	NoConnectionRetry,                          // OBSOLETE - removed
	InvalidPassword,                            // password/ticket is invalid
	LoggedInElsewhere,                          // same user logged in elsewhere
	InvalidProtocolVer,                         // protocol version is incorrect
	InvalidParam,                               // a parameter is incorrect
	FileNotFound,                               // file was not found
	Busy,                                       // called method busy - action not taken
	InvalidState,                               // called object was in an invalid state
	InvalidName,                                // name is invalid
	InvalidEmail,                               // email is invalid
	DuplicateName,                              // name is not unique
	AccessDenied,                               // access is denied
	Timeout,                                    // operation timed out
	Banned,                                     // VAC2 banned
	AccountNotFound,                            // account not found
	InvalidSteamID,                             // steamID is invalid
	ServiceUnavailable,                         // The requested service is currently unavailable
	NotLoggedOn,                                // The user is not logged on
	Pending,                                    // Request is pending (may be in process, or waiting on third party)
	EncryptionFailure,                          // Encryption or Decryption failed
	InsufficientPrivilege,                      // Insufficient privilege
	LimitExceeded,                              // Too much of a good thing
	Revoked,                                    // Access has been revoked (used for revoked guest passes)
	Expired,                                    // License/Guest pass the user is trying to access is expired
	AlreadyRedeemed,                            // Guest pass has already been redeemed by account, cannot be acked again
	DuplicateRequest,                           // The request is a duplicate and the action has already occurred in the past, ignored this time
	AlreadyOwned,                               // All the games in this guest pass redemption request are already owned by the user
	IPNotFound,                                 // IP address not found
	PersistFailed,                              // failed to write change to the data store
	LockingFailed,                              // failed to acquire access lock for this operation
	LogonSessionReplaced,                       //
	ConnectFailed,                              //
	HandshakeFailed,                            //
	IOFailure,                                  //
	RemoteDisconnect,                           //
	ShoppingCartNotFound,                       // failed to find the shopping cart requested
	Blocked,                                    // a user didn't allow it
	Ignored,                                    // target is ignoring sender
	NoMatch,                                    // nothing matching the request found
	AccountDisabled,                            //
	ServiceReadOnly,                            // this service is not accepting content changes right now
	AccountNotFeatured,                         // account doesn't have value, so this feature isn't available
	AdministratorOK,                            // allowed to take this action, but only because requester is admin
	ContentVersion,                             // A Version mismatch in content transmitted within the Steam protocol.
	TryAnotherCM,                               // The current CM can't service the user making a request, user should try another.
	PasswordRequiredToKickSession,              // You are already logged in elsewhere, this cached credential login has failed.
	AlreadyLoggedInElsewhere,                   // You are already logged in elsewhere, you must wait
	Suspended,                                  // Long running operation (content download) suspended/paused
	Cancelled,                                  // Operation canceled (typically by user: content download)
	DataCorruption,                             // Operation canceled because data is ill formed or unrecoverable
	DiskFull,                                   // Operation canceled - not enough disk space.
	RemoteCallFailed,                           // an remote call or IPC call failed
	PasswordUnset,                              // Password could not be verified as it's unset server side
	ExternalAccountUnlinked,                    // External account (PSN, Facebook...) is not linked to a Steam account
	PSNTicketInvalid,                           // PSN ticket was invalid
	ExternalAccountAlreadyLinked,               // External account (PSN, Facebook...) is already linked to some other account, must explicitly request to replace/delete the link first
	RemoteFileConflict,                         // The sync cannot resume due to a conflict between the local and remote files
	IllegalPassword,                            // The requested new password is not legal
	SameAsPreviousValue,                        // new value is the same as the old one ( secret question and answer )
	AccountLogonDenied,                         // account login denied due to 2nd factor authentication failure
	CannotUseOldPassword,                       // The requested new password is not legal
	InvalidLoginAuthCode,                       // account login denied due to auth code invalid
	AccountLogonDeniedNoMail,                   // account login denied due to 2nd factor auth failure - and no mail has been sent - partner site specific
	HardwareNotCapableOfIPT,                    // 
	IPTInitError,                               // 
	ParentalControlRestricted,                  // operation failed due to parental control restrictions for current user
	FacebookQueryError,                         // Facebook query returned an error
	ExpiredLoginAuthCode,                       // account login denied due to auth code expired
	IPLoginRestrictionFailed,                   //
	AccountLockedDown,                          //
	AccountLogonDeniedVerifiedEmailRequired,    //
	NoMatchingURL,                              //
	BadResponse,                                // parse failure, missing field, etc.
	RequirePasswordReEntry,                     // The user cannot complete the action until they re-enter their password
	ValueOutOfRange,                            // the value entered is outside the acceptable range
	UnexpectedError,                            // something happened that we didn't expect to ever happen
	Disabled,                                   // The requested service has been configured to be unavailable
	InvalidCEGSubmission,                       // The set of files submitted to the CEG server are not valid !
	RestrictedDevice,                           // The device being used is not allowed to perform this action
	RegionLocked,                               // The action could not be complete because it is region restricted
	RateLimitExceeded,                          // Temporary rate limit exceeded, try again later, different from k_EResultLimitExceeded which may be permanent
	AccountLoginDeniedNeedTwoFactor,            // Need two-factor code to login
	ItemDeleted,                                // The thing we're trying to access has been deleted
	AccountLoginDeniedThrottle,                 // login attempt failed, try to throttle response to possible attacker
	TwoFactorCodeMismatch,                      // two factor code mismatch
	TwoFactorActivationCodeMismatch,            // activation code for two-factor didn't match
	AccountAssociatedToMultiplePartners,        // account has been associated with multiple partners
	NotModified,                                // data not modified
	NoMobileDevice,                             // the account does not have a mobile device associated with it
	TimeNotSynced,                              // the time presented is out of range or tolerance
	SmsCodeFailed,                              // SMS code failure (no match, none pending, etc.)
	AccountLimitExceeded,                       // Too many accounts access this resource
	AccountActivityLimitExceeded,               // Too many changes to this account
	PhoneActivityLimitExceeded,                 // Too many changes to this phone
	RefundToWallet,                             // Cannot refund to payment method, must use wallet
	EmailSendFailure,                           // Cannot send an email
	NotSettled,                                 // Can't perform operation till payment has settled
	NeedCaptcha,                                // Needs to provide a valid captcha
	GSLTDenied,                                 // a game server login token owned by this token's owner has been banned
	GSOwnerDenied,                              // game server owner is denied for other reason (account lock, community ban, vac ban, missing phone)
	InvalidItemType,                            // the type of thing we were requested to act on is invalid
	IPBanned,                                   // the ip address has been banned from taking this action
	GSLTExpired,                                // this token has expired from disuse; can be reset for use
	InsufficientFunds,                          // user doesn't have enough wallet funds to complete the action
	TooManyPending,                             // There are too many of this thing pending already
	NoSiteLicensesFound,                        // No site licenses found
	WGNetworkSendExceeded,                      // the WG couldn't send a response because we exceeded max network send size
	AccountNotFriends,                          // the user is not mutually friends
	LimitedUserAccount,                         // the user is limited
	CantRemoveItem,                             // item can't be removed
	AccountDeleted,                             // account has been deleted
	ExistingUserCancelledLicense,               // A license for this already exists, but cancelled
	CommunityCooldown,                          // access is denied because of a community cooldown (probably from support profile data resets)
	NoLauncherSpecified,                        // No launcher was specified, but a launcher was needed to choose correct realm for operation.
	MustAgreeToSSA,                             // User must agree to china SSA or global SSA before login
	LauncherMigrated,                           // The specified launcher type is no longer supported; the user should be directed elsewhere
	SteamRealmMismatch,                         // The user's realm does not match the realm of the requested resource
	InvalidSignature,                           // signature check did not match
	ParseFailure,                               // Failed to parse input
	NoVerifiedPhone,                            // account does not have a verified phone number
	InsufficientBattery,                        // user device doesn't have enough battery charge currently to complete the action
	ChargerRequired,                            // The operation requires a charger to be plugged in, which wasn't present
	CachedCredentialInvalid,                    // Cached credential was invalid - user must reauthenticate
	PhoneNumberIsVOIP,                          // The phone number provided is a Voice Over IP number
	NotSupported,                               // The data being accessed is not supported by this API
	FamilySizeLimitExceeded,                    // Reached the maximum size of the family
	OfflineAppCacheInvalid                      //
}

pub enum EBeginAuthSessionResult {
	Ok,						// Ticket is valid for this game and this steamID.
	InvalidTicket,			// Ticket is not valid.
	DuplicateRequest,		// A ticket has already been submitted for this steamID
	InvalidVersion,			// Ticket is from an incompatible interface version
	GameMismatch,			// Ticket is not for this game
	ExpiredTicket
}

pub enum EServerMode {
	Invalid,                 // DO NOT USE
	NoAuthentication,        // Don't authenticate user logins and don't list on the server list
	Authentication,          // Authenticate users, list on the server list, don't run VAC on clients that connect
	AuthenticationAndSecure, // Authenticate users, list on the server list and VAC protect clients
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

impl Context {
	pub fn create_interface<T>(&self) -> Option<T>
	where
		T: interface::Interface {
		let version_ptr = T::VERSION.as_ptr();
		//assert!(version_ptr as usize % 0x4 == 0, "version_ptr 0x{:#} Pointer not aligned", version_ptr.addr());
		//assert!(self.c_fn_create_interface as usize % 0x4 == 0, "c_fn_create_interface Pointer not aligned");

		let obj = unsafe { (self.c_fn_create_interface)(self.user.0, T::VERSION.as_ptr() ) };
		if obj.is_null() || obj.addr() <= 0x10000 {
			None
		} else {
			Some(T::create(obj, self.clone()))
		}
	}
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

	pub fn is_valid(&self) -> bool {
		self.account_type().is_some() && self.universe().is_some()
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

impl EServerMode {
	pub(crate) fn v1_into(&self) -> Option<u32> {
		match self {
			EServerMode::Invalid => Some(0),
			EServerMode::NoAuthentication => Some(1),
			EServerMode::Authentication => Some(2),
			EServerMode::AuthenticationAndSecure => Some(3),
		}
	}
}

impl std::fmt::Display for AccountType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			AccountType::Invalid => 'I',
			AccountType::Individual => 'U',
			AccountType::Multiseat => 'M',
			AccountType::GameServer => 'G',
			AccountType::AnonGameServer => 'A',
			AccountType::Pending => 'P',
			AccountType::ContentServer => 'C',
			AccountType::Clan => 'g',
			AccountType::Chat => 'T',
			AccountType::P2P => '2',
			AccountType::AnonUser => 'a',
		})
	}
}

impl std::fmt::Display for CSteamID {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_valid() {
			match f.width() {
				Some(3) => write!(f, "[{}:{}:{}]", self.account_type().unwrap(), self.universe().unwrap() as u8, self.account_id()),
				Some(2) => write!(f, "STEAM_{}:{}:{}", self.universe().unwrap() as u8, (self.account_id() & 0b1), (self.account_id() << 1)),
				_ => write!(f, "{}", self.0)
			}
		} else {
			write!(f, "INVALID_STEAM_ID")
		}
	}
}