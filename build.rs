use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    // Only provide memory.x for ARM Cortex-M targets
    if target.starts_with("thumbv") {
        let memory_x = if target == "thumbv7em-none-eabihf" {
            // STM32H753ZI (Cortex-M7)
            // Flash: 2MB at 0x08000000
            // RAM: Multiple banks totaling 1MB
            //   - DTCMRAM: 128KB at 0x20000000 (fastest, tightly coupled)
            //   - RAM_D1: 512KB at 0x24000000 (AXI SRAM)
            //   - RAM_D2: 288KB at 0x30000000 (AHB SRAM)
            //   - RAM_D3: 64KB at 0x38000000 (backup domain)
            r#"
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 2048K
  RAM   : ORIGIN = 0x24000000, LENGTH = 512K
}
"#
        } else {
            return; // No memory.x needed for other targets
        };

        fs::write(out_dir.join("memory.x"), memory_x).unwrap();
        println!("cargo:rustc-link-search={}", out_dir.display());
    }
}
