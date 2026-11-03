use super::*;

fn check(spec: &str) {
    let names = ["A", "a", "=", "+", "Minus", "ArrowLeft", "Left", "Delete", "Backspace", "Space", "F12", "?", "é", "Menu"];
    assert_eq!(normalize(spec), legacy::normalize(spec), "normalize {spec:?}");
    let old = legacy::parse(spec).map(|(mods, key)| ((mods.shift, mods.alt, mods.cmd, mods.ctrl), key));
    let new = parse(spec).map(|(mods, key)| ((mods.shift, mods.alt, mods.cmd, mods.ctrl), key));
    assert_eq!(new, old, "parse {spec:?}");
    for platform in [Platform::Mac, Platform::Other] {
        assert_eq!(display(spec, platform), legacy::display(spec, platform), "display {spec:?}");
        for name in names {
            for bits in 0..16 {
                let mods = Mods { shift: bits & 1 != 0, alt: bits & 2 != 0, cmd: bits & 4 != 0, ctrl: bits & 8 != 0 };
                let old_mods = legacy::Mods { shift: mods.shift, alt: mods.alt, cmd: mods.cmd, ctrl: mods.ctrl };
                assert_eq!(
                    matches(spec, platform, name, mods),
                    legacy::matches(spec, platform, name, old_mods),
                    "matches {spec:?} {name:?} {mods:?}"
                );
                assert_eq!(text_motion(platform, name, mods), legacy::text_motion(platform, name, old_mods));
            }
        }
    }
}

#[test]
fn default_and_gesture_identity() {
    for binding in KEYS {
        for spec in binding.mac.iter().chain(binding.other) {
            check(spec);
        }
    }
    for (_, _, mac, other) in GESTURES {
        check(mac);
        check(other);
    }
}

#[test]
fn seeded_spec_identity() {
    let tokens =
        ["Shift", "Cmd", "Ctrl", "Alt", "Option", "Plus", "Minus", "F1", "F12", "ArrowLeft", "Space", "Escape", "?", "=", ";", "é", "", "Menu"];
    let mut state = 0x76_u64;
    for _ in 0..2000 {
        let mut spec = String::new();
        for index in 0..3 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            if index > 0 {
                spec.push('+');
            }
            spec.push_str(tokens[((state >> 32) as usize) % tokens.len()]);
        }
        check(&spec);
    }
}
