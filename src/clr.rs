extern crate libloading as lib;

use std;
use std::ffi::{CString, OsStr};

use crate::hresult::*;
use crate::error::{IronCoreError, IronCoreResult};
use std::path::{Path, PathBuf};

pub enum CoreClrHostHandle {}
pub type CoreClrDomainId = u32;
pub type CoreClrString = *const std::os::raw::c_char;
pub type CoreClrDelegatePointer = *const ();

type CoreClrInitializeFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    exe_path: CoreClrString,
    app_domain_friendly_name: CoreClrString,
    property_count: isize,
    property_keys: *const CoreClrString,
    property_values: *const CoreClrString,
    host_handle: *mut *mut CoreClrHostHandle,
    domain_id: *mut CoreClrDomainId) -> u32>;

type CoreClrExecuteAssemblyFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    host_handle: *const CoreClrHostHandle,
    domain_id: CoreClrDomainId,
    argc: isize,
    argv: *const CoreClrString,
    managed_assembly_path: CoreClrString,
    exit_code: *mut u32) -> u32>;

type CoreClrCreateDelegateFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    host_handle: *const CoreClrHostHandle,
    domain_id: CoreClrDomainId,
    entry_point_assembly_name: CoreClrString,
    entry_point_type_name: CoreClrString,
    entry_point_method_name: CoreClrString,
    delegate: *mut *const ()) -> u32>;

type CoreClrShutdownFn<'a> = lib::Symbol<'a, unsafe extern "C" fn(
    host_handle: *const CoreClrHostHandle,
    domain_id: CoreClrDomainId) -> u32>;

pub struct CoreClrInstance<'a> {
    symbols: CoreClrSymbols<'a>,
    libclr: lib::Library,
    clr_host_handle: *const CoreClrHostHandle,
    clr_domain_id: CoreClrDomainId,
}

struct CoreClrSymbols<'lib> {
    pub coreclr_initialize: CoreClrInitializeFn<'lib>,
    pub coreclr_execute_assembly: CoreClrExecuteAssemblyFn<'lib>,
    pub coreclr_create_delegate: CoreClrCreateDelegateFn<'lib>,
    pub coreclr_shutdown: CoreClrShutdownFn<'lib>
}

impl<'lib> CoreClrSymbols<'lib> {
    pub fn new(clr: &'lib lib::Library) -> IronCoreResult<Self> {
        Ok(Self {
            coreclr_initialize: unsafe { clr.get(b"coreclr_initialize\0")? },
            coreclr_execute_assembly: unsafe { clr.get(b"coreclr_execute_assembly\0")? },
            coreclr_create_delegate: unsafe { clr.get(b"coreclr_create_delegate\0")? },
            coreclr_shutdown: unsafe { clr.get(b"coreclr_shutdown\0")? },
        })
    }
}

impl<'a> CoreClrInstance<'a> {
    pub fn new<V: AsRef<str>>(version: V, app_paths: String, app_ni_paths: String, native_dll_search_dirs: String) -> IronCoreResult<CoreClrInstance<'a>> {
        let libclr = load_coreclr_library(version.as_ref())?;
        let symbols = CoreClrSymbols::new(&libclr)?;

        let exe = std::env::current_exe()?;
        let exe_str = exe.to_str().ok_or(IronCoreError::InvalidExePath)?;
        let clr_exe_path = CString::new(exe_str)?;
        let clr_app_domain_friendly_name = CString::new("Rust CLR Host")?;
        let clr_trusted_asms = get_trusted_assemblies(version)?;

        let property_keys: Vec<&str> = vec!["TRUSTED_PLATFORM_ASSEMBLIES", "APP_PATHS", "APP_NI_PATHS", "NATIVE_DLL_SEARCH_DIRECTORIES", "AppDomainCompatSwitch"];
        let (_clr_property_keys, clr_property_keys_ptr) = vec2cstring(property_keys)?;

        let property_values: Vec<&str> = vec![&clr_trusted_asms, app_paths.as_str(), app_ni_paths.as_str(), native_dll_search_dirs.as_str(), "UseLatestBehaviorWhenTFMNotSpecified"];
        let (_clr_property_values, clr_property_values_ptr) = vec2cstring(property_values)?;

        let mut clr_host_handle: *mut CoreClrHostHandle = std::ptr::null_mut();
        let clr_host_handle_ptr: *mut *mut CoreClrHostHandle = &mut clr_host_handle;

        let mut clr_domain_id: CoreClrDomainId = 0u32;
        let clr_domain_id_ptr: *mut CoreClrDomainId = &mut clr_domain_id;

        let fun = &symbols.coreclr_initialize;
        let hr = HRESULT::from(fun(
            clr_exe_path.as_ptr(),
            clr_app_domain_friendly_name.as_ptr(),
            clr_property_keys_ptr.len() as isize,
            clr_property_keys_ptr.as_ptr(),
            clr_property_values_ptr.as_ptr(),
            clr_host_handle_ptr,
            clr_domain_id_ptr,
        ));
        hr.check()?;

        Ok(Self {
            libclr,
            symbols,
            clr_domain_id,
            clr_host_handle
        })
    }

    pub fn execute_assembly(&self, assembly: &str, args: Vec<&str>) -> IronCoreResult<u32> {
        unsafe {
            let clr_assembly = CString::new(assembly)?;

            let (_clr_args, clr_args_ptr) = vec2cstring(args)?;

            let mut clr_exit_code = 0u32;
            let clr_exit_code_ptr: *mut u32 = &mut clr_exit_code;

            let coreclr_execute_assembly = &self.symbols.coreclr_execute_assembly;
            let hr = HRESULT::from(coreclr_execute_assembly(
                self.clr_host_handle,
                self.clr_domain_id,
                clr_args_ptr.len() as isize,
                clr_args_ptr.as_ptr(),
                clr_assembly.as_ptr(),
                clr_exit_code_ptr
            ));
            hr.check()?;

            return Ok(clr_exit_code);
        }
    }

    pub fn create_delegate(&self, entry_point_assembly_name: &str, entry_point_type_name: &str, entry_point_method_name: &str) -> IronCoreResult<CoreClrDelegatePointer> {
        let clr_entry_point_assembly_name = CString::new(entry_point_assembly_name)?;
        let clr_entry_point_type_name = CString::new(entry_point_type_name)?;
        let clr_entry_point_method_name = CString::new(entry_point_method_name)?;

        unsafe {
            let mut delegate = core::mem::MaybeUninit::<CoreClrDelegatePointer>::uninit();;
            let delegate_ptr = delegate.as_mut_ptr();

            let coreclr_create_delegate = &self.symbols.coreclr_create_delegate;
            let hr = HRESULT::from(coreclr_create_delegate(
                self.clr_host_handle,
                self.clr_domain_id,
                clr_entry_point_assembly_name.as_ptr(),
                clr_entry_point_type_name.as_ptr(),
                clr_entry_point_method_name.as_ptr(),
                delegate_ptr
            ));
            hr.check()?;

            return Ok(delegate.assume_init());
        }
    }
}

impl<'a> Drop for CoreClrInstance<'a> {
    fn drop(&mut self) {
        unsafe {
            let coreclr_shutdown = &self.symbols.coreclr_shutdown;
            let _ = HRESULT::from(coreclr_shutdown(self.clr_host_handle, self.clr_domain_id));
        }
    }
}

fn vec2cstring(strings: Vec<&str>) -> IronCoreResult<(Vec<CString>, Vec<CoreClrString>)> {
    let strings_result: std::result::Result<Vec<CString>, std::ffi::NulError> =
        strings
            .clone()
            .into_iter()
            .map(|x| CString::new(x))
            .collect();
    let cstrings = strings_result?;
    let cstrings_ptr: Vec<CoreClrString> =
        cstrings.iter().map(|x| x.as_ptr()).collect();

    return Ok((cstrings, cstrings_ptr));
}

fn get_runtime_dir<V: AsRef<str>>(version: V) -> IronCoreResult<PathBuf> {

    // TODO: Very Ubuntu Linux specific
    let dir = Path::new("/usr")
        .join("share")
        .join("dotnet")
        .join("shared")
        .join("Microsoft.NETCore.App")
        .join(version.as_ref());

    return Ok(dir);
}

fn get_runtime_path<V: AsRef<str>>(version: V) -> IronCoreResult<PathBuf> {
    let mut dll = get_runtime_dir(version)?.clone();

    // TODO: Very Ubuntu Linux specific
    dll = dll.join("libcoreclr.so");

    return Ok(dll);
}

fn get_trusted_assemblies<V: AsRef<str>>(version: V) -> IronCoreResult<String> {
    let mut result = String::new();

    let coreclr_dir = get_runtime_dir(version)?;
    let coreclr_files = std::fs::read_dir(coreclr_dir)?;
    for file in coreclr_files {
        let file = file?;
        let filepath = file.path();
        if let Some(fileext) = filepath.extension().and_then(OsStr::to_str) {
            if fileext == "dll" {
                if result.len() > 0 {
                    result.push_str(";");
                }
                if let Some(filepath_str) = filepath.to_str() {
                    result.push_str(filepath_str);
                }
            }
        }
    }

    return Ok(result);
}

pub fn load_coreclr_library<V: AsRef<str>>(version: V) -> IronCoreResult<lib::Library> {
    let libclr_path = get_runtime_path(version)?;
    let libclr = lib::Library::new(std::ffi::OsString::from(libclr_path))?;
    return Ok(libclr);
}
