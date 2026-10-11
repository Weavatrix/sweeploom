//! Xcode's DerivedData folder suffix: MD5 of the workspace path as 28 base-26 letters.

/// `Name-<suffix>` for a `.xcodeproj`/`.xcworkspace` path, as Xcode names DerivedData folders.
pub(super) fn derived_suffix(workspace: &str) -> String {
    let digest = md5(workspace.as_bytes());
    let mut out = [b'a'; 28];
    for (half, slots) in out.chunks_mut(14).enumerate() {
        let mut value = u64::from_be_bytes(
            digest[half * 8..half * 8 + 8]
                .try_into()
                .unwrap_or_default(),
        );
        for slot in slots.iter_mut().rev() {
            *slot = b'a' + (value % 26) as u8;
            value /= 26;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(super) fn md5(input: &[u8]) -> [u8; 16] {
    const SHIFT: [u32; 16] = [7, 12, 17, 22, 5, 9, 14, 20, 4, 11, 16, 23, 6, 10, 15, 21];
    let table: Vec<u32> = (1..=64)
        .map(|index| (f64::from(index).sin().abs() * 4_294_967_296.0) as u32)
        .collect();
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((input.len() as u64).wrapping_mul(8)).to_le_bytes());
    let mut state: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    for chunk in message.chunks_exact(64) {
        let words: Vec<u32> = chunk
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect();
        let [mut a, mut b, mut c, mut d] = state;
        for step in 0..64 {
            let (mix, index) = match step / 16 {
                0 => ((b & c) | (!b & d), step),
                1 => ((d & b) | (!d & c), (5 * step + 1) % 16),
                2 => (b ^ c ^ d, (3 * step + 5) % 16),
                _ => (c ^ (b | !d), (7 * step) % 16),
            };
            let rotated = mix
                .wrapping_add(a)
                .wrapping_add(table[step])
                .wrapping_add(words[index])
                .rotate_left(SHIFT[step / 16 * 4 + step % 4]);
            (a, d, c) = (d, c, b);
            b = b.wrapping_add(rotated);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut out = [0u8; 16];
    for (bytes, word) in out.chunks_exact_mut(4).zip(state) {
        bytes.copy_from_slice(&word.to_le_bytes());
    }
    out
}
