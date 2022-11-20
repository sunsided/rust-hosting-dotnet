mod error;
pub mod hresult;
pub mod clr;

fn main() {
    let version = "7.0.0";

    let current_dir = std::env::current_dir().unwrap();

    let coreclr_app_paths = format!("{}", current_dir.display());
    let coreclr_ni_app_paths = coreclr_app_paths.clone();
    let coreclr_dll_native_search_dirs = coreclr_app_paths.clone();

    let coreclr = clr::CoreClrInstance::new(version, coreclr_app_paths, coreclr_ni_app_paths, coreclr_dll_native_search_dirs).unwrap();

    coreclr.execute_assembly("ironcore-example.dll", Vec::default()).unwrap();

    /*
    unsafe {
        let delegate_ptr = coreclr.create_delegate("ironcore-example", "IronCore.Example.Scripts", "Main").unwrap();
        let delegate = std::mem::transmute::<clr::CoreClrDelegatePointer, extern "system" fn() -> ()>(delegate_ptr);
        delegate();
    }
    */
}
