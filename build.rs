extern crate winres;

#[path = "src/service/properties.rs"]
pub mod properties;

fn main() {
    let mut res = winres::WindowsResource::new();

    // set display properties
    res.set("FileDescription", properties::SERVICE_DISPLAY_NAME);
    res.set("ProductName", properties::SERVICE_DISPLAY_NAME);

    // set version properties
    let major_minor_patch = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = str::split(major_minor_patch, '.').collect();
    if let [major_string, minor_string, patch_string] = *parts {
        let major: u64 = str::parse(major_string).unwrap();
        let minor: u64 = str::parse(minor_string).unwrap();
        let patch: u64 = str::parse(patch_string).unwrap();
        let revision: u64 = str::parse(option_env!("BUILD_NUMBER").unwrap_or("0")).unwrap();
        
        let version_string = format!("{}.{}.{}.{}", major, minor, patch, revision);        
        res.set("ProductVersion", &version_string);
        res.set("FileVersion", &version_string);        
     
        let mut version_binary = 0 as u64;
        version_binary |= major << 48;
        version_binary |= minor << 32;
        version_binary |= patch << 16;
        version_binary |= revision;
        res.set_version_info(winres::VersionInfo::FILEVERSION, version_binary);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_binary);
    }

    // require elevation so that we can perform SCM ops
    if std::env::var("PROFILE").unwrap() == "release" {
        res.set_manifest(r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
            <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
                <security>
                    <requestedPrivileges>
                        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
                    </requestedPrivileges>
                </security>
            </trustInfo>
        </assembly>
        "#);
    }

    // embed icon
    res.set_icon("arwad.ico");

    res.compile().unwrap();
}