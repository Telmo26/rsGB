mod mooneye_tests {
    use std::collections::HashMap;

use rsgb_core::{AudioSink, VideoSink};
    struct TestAudioSink {}
    impl AudioSink for TestAudioSink {
        fn push_sample(&mut self, _left: f32, _right: f32) {
            ()
        }
    }

    struct TestVideoSink {
        framebuffer: [u32; 0x5A00]
    }
    
    impl VideoSink for TestVideoSink {
        fn get_mut(&mut self) -> &mut [u32] {
           &mut self.framebuffer
        }

        fn present(&mut self) {
            ()
        }
    }

    mod acceptance {
        use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

        use rsgb_core::{Gameboy, settings::Settings};

use crate::mooneye_tests::{TestAudioSink, TestVideoSink};

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
            let audio_sink = TestAudioSink {};
            let video_sink = TestVideoSink { framebuffer: [0; 0x5A00] };

            let mut gb = Gameboy::new(
                rsgb_core::ColorMode::ARGB, 
                audio_sink,
                video_sink
            );

            let settings = Settings::default();
            let rom_path = PathBuf::from(path);

            if SKIP_LIST.contains(&rom_path.file_stem().unwrap().to_str().unwrap()) {
                return
            }

            gb.load_cartridge(&rom_path, &settings);

            let timeout = Duration::from_secs(20);
            let start_time = Instant::now();

            while start_time.elapsed() < timeout && !gb.debug().current_instruction().contains("JR FE") { // Infinite loop of jumping in place
                gb.next_frame(&settings);
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