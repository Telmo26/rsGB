mod mooneye_tests {
    use std::collections::HashMap;

    mod acceptance {
        use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

        use rs_gb_core::{Gameboy, settings::Settings};

        const SKIP_LIST: [&str; 9] = [
            "boot_div2-S",
            "boot_div-S",
            "boot_div-dmg0",
            "boot_hwio-S",
            "boot_hwio-dmg0",
            "boot_regs-dmg0",
            "boot_regs-mgb",
            "boot_regs-sgb",
            "boot_regs-sgb2",
        ];

        #[test_each::blob(glob = "test_roms/mooneye/acceptance/**/*.gb", name(segments = 1))]
        fn run_test(_content: &[u8], path: &Path) {
            let mut gb = Gameboy::new(rs_gb_core::ColorMode::ARGB, |_| {});

            let settings = Settings::default();
            let rom_path = PathBuf::from(path);

            if SKIP_LIST.contains(&rom_path.file_stem().unwrap().to_str().unwrap()) {
                return
            }

            gb.load_cartridge(&rom_path, &settings);

            let timeout = Duration::from_secs(20);
            let start_time = Instant::now();

            let mut framebuffer = [0; 0x5A00];

            while start_time.elapsed() < timeout && !gb.debug().current_instruction().contains("JR FE") { // Infinite loop of jumping in place
                gb.next_frame(&mut framebuffer, &settings);
            }
            
            let debug_info = gb.debug();
            let registers = debug_info.registers();
            assert!(start_time.elapsed() < timeout);
            assert!(super::successful_test(&registers));
        }
    }
    

    fn successful_test(registers: &HashMap<&str, u16>) -> bool {
        return (registers["b"] == 3) &
            (registers["c"] == 5) &
            (registers["d"] == 8) &
            (registers["e"] == 13) &
            (registers["h"] == 21) &
            (registers["l"] == 34)
    }
}