use std::path::Path;

fn main() {
    // 用「无默认 manifest」的 Windows 属性构建，manifest 改由下方统一嵌入。
    // 否则 tauri-build 会把 MANIFEST(1) 资源打进 resource.lib，与下方针对测试二进制
    // 的 /MANIFESTINPUT 重复（CVTRES CVT1100：资源重复 类型 MANIFEST 名称 1）。
    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attrs).expect("failed to run tauri-build");

    // 统一给所有目标（主 bin、cdylib、测试二进制）嵌入 Common-Controls v6 manifest。
    // 修复：Windows 上 `cargo test` 的测试二进制缺少 manifest，导致 tao 静态链接的
    // comctl32.dll v6 未被激活，启动即 STATUS_ENTRYPOINT_NOT_FOUND（tauri#13419）。
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let manifest_path = Path::new(&out_dir).join("windows-app-manifest.xml");
        std::fs::write(&manifest_path, WINDOWS_APP_MANIFEST).unwrap();

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo::rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
    }
}

/// 与 tauri-build 默认的 windows-app-manifest.xml 内容一致：声明对
/// Microsoft.Windows.Common-Controls v6 的依赖。
const WINDOWS_APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;
