pub fn hash(data: &[u8], index: usize, length: usize, seed: u32) -> u32 {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;

    let init = 0xDEADBEEF + length as u32 + seed;
    a = init;
    b = init;
    c = init;

    let mut i = index;
    while i + 12 < length {
        a = a.wrapping_add(
            data[i] as u32
                | ((data[i + 1] as u32) << 8)
                | ((data[i + 2] as u32) << 16)
                | ((data[i + 3] as u32) << 24),
        );
        i += 4;
        b = b.wrapping_add(
            data[i] as u32
                | ((data[i + 1] as u32) << 8)
                | ((data[i + 2] as u32) << 16)
                | ((data[i + 3] as u32) << 24),
        );
        i += 4;
        c = c.wrapping_add(
            data[i] as u32
                | ((data[i + 1] as u32) << 8)
                | ((data[i + 2] as u32) << 16)
                | ((data[i + 3] as u32) << 24),
        );
        i += 4;

        a = a.wrapping_sub(c);
        a ^= (c << 4) | (c >> (32 - 4));
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= (a << 6) | (a >> (32 - 6));
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= (b << 8) | (b >> (32 - 8));
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c);
        a ^= (c << 16) | (c >> (32 - 16));
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= (a << 19) | (a >> (32 - 19));
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= (b << 4) | (b >> (32 - 4));
        b = b.wrapping_add(a);
    }

    if i < length {
        a = a.wrapping_add(data[i] as u32);
        i += 1;
    }

    if i < length {
        a = a.wrapping_add((data[i] as u32) << 8);
        i += 1;
    }

    if i < length {
        a = a.wrapping_add((data[i] as u32) << 16);
        i += 1;
    }

    if i < length {
        a = a.wrapping_add((data[i] as u32) << 24);
        i += 1;
    }

    if i < length {
        b = b.wrapping_add(data[i] as u32);
        i += 1;
    }

    if i < length {
        b = b.wrapping_add((data[i] as u32) << 8);
        i += 1;
    }

    if i < length {
        b = b.wrapping_add((data[i] as u32) << 16);
        i += 1;
    }

    if i < length {
        b = b.wrapping_add((data[i] as u32) << 24);
        i += 1;
    }

    if i < length {
        c = c.wrapping_add(data[i] as u32);
        i += 1;
    }

    if i < length {
        c = c.wrapping_add((data[i] as u32) << 8);
        i += 1;
    }

    if i < length {
        c = c.wrapping_add((data[i] as u32) << 16);
        i += 1;
    }

    if i < length {
        c = c.wrapping_add((data[i] as u32) << 24);
        // i += 1;
    }

    c ^= b;
    c = c.wrapping_sub((b << 14) | (b >> (32 - 14)));
    a ^= c;
    a = a.wrapping_sub((c << 11) | (c >> (32 - 11)));
    b ^= a;
    b = b.wrapping_sub((a << 25) | (a >> (32 - 25)));
    c ^= b;
    c = c.wrapping_sub((b << 16) | (b >> (32 - 16)));
    a ^= c;
    a = a.wrapping_sub((c << 4) | (c >> (32 - 4)));
    b ^= a;
    b = b.wrapping_sub((a << 14) | (a >> (32 - 14)));
    c ^= b;
    c = c.wrapping_sub((b << 24) | (b >> (32 - 24)));

    return c;
}

pub fn hash_string(str: &str) -> u32 {
    let bytes = str.as_bytes();
    hash(bytes, 0, bytes.len(), 0)
}

#[test]
fn sanity() {
    hash(&[], 0, 0, 0);
    hash_string(&"");
}

#[test]
fn hash_compliance() {
    assert_eq!(hash_string(&"ui/intro.gfx"), 2386027578);
    assert_eq!(hash_string(&"rico rodriguez :3"), 1080157782);
}
