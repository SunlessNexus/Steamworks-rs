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

pub fn init(path: std::path::PathBuf) -> Result<Context, InitError> {
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

	let init = module.find(b"SteamAPI_InitFlat\0");
	if init.is_null() {
		return Err(InitError::FailedInit)
	}

	let shutdown = module.find(b"SteamAPI_Shutdown\0");
	if shutdown.is_null() {
		return Err(InitError::FailedInit)
	}

	let init: unsafe extern "C" fn(*const ()) -> u8 = unsafe { std::mem::transmute(init) };
	unsafe {
		if init(std::ptr::null()) != 0x0 {
			return Err(InitError::FailedInit)
		}
	}

	Ok(Context {
		library: module,
		c_fn_shutdown: unsafe { std::mem::transmute(shutdown) }
	})
}