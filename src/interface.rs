pub trait Interface {
	const VERSION: &'static str;
	fn object_ptr(&self) -> *mut std::ffi::c_void;
    fn linked_context(&self) -> &crate::Context;
	fn create(object_ptr: *mut std::ffi::c_void, context: crate::Context) -> Self;
}

macro_rules! ifunc {
    ($funcname:ident,
     $win_off:literal,
     $linux_off:literal,
     $return_ty:ty,
     ($($param_name:ident: $param_ty:ty),*) ) => {
	#[allow(non_snake_case)]
	pub fn $funcname<I>(obj: &I, $($param_name: $param_ty),*) -> $return_ty
	where
		I: Interface
	{

		unsafe {
			let vtable: *mut *mut *mut std::ffi::c_void = std::mem::transmute(obj.object_ptr());
			let vtable: *mut *mut std::ffi::c_void = *vtable;

			#[cfg(windows)]
			let func: *mut std::ffi::c_void = *vtable.add($win_off);
			#[cfg(unix)]
			let func: *mut std::ffi::c_void = *vtable.add($linux_off);

			#[cfg(all(target_os = "windows", target_arch = "x86"))]
			let func: unsafe extern "thiscall" fn(*mut std::ffi::c_void, $($param_ty),*) -> $return_ty = std::mem::transmute(func);
			#[cfg(not(all(target_os = "windows", target_arch = "x86")))]
			let func: unsafe extern "C" fn(*mut std::ffi::c_void, $($param_ty),*) -> $return_ty = std::mem::transmute(func);

			return func(obj.object_ptr(), $($param_name),*);
		}
	}
}}

pub(crate) use ifunc;

macro_rules! impl_steam_iface {
    ($($variant:ident($data:ty)),*) => {
        pub(crate) trait InterfaceUnwrap<T>
		where
			T: Interface
		{
            fn unwrap(self) -> T;
        }

        enum EInterface {
            $(
                $variant($data),
            )*
        }

        $(
            impl InterfaceUnwrap<$data> for EInterface {
                fn unwrap(self) -> $data {
                    match self {
                        EInterface::$variant(iface) => iface,
                        _ => panic!("Wrong Interface type!")
                    }
                }
            }
        )*
    }
}

impl_steam_iface!(
    ISteamUserV23(crate::steamuser::v23::ISteamUser)
);