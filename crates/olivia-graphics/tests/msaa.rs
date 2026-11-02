//! How a requested MSAA sample count meets what the GPU supports.

use olivia_graphics::sample_count;

const SUPPORTED: [u32; 3] = [1, 2, 4];

#[test]
fn a_supported_count_is_kept() {
    assert_eq!(sample_count(4, &SUPPORTED), 4);
}

#[test]
fn one_disables_msaa() {
    assert_eq!(sample_count(1, &SUPPORTED), 1);
}

#[test]
fn an_unsupported_count_falls_to_the_nearest_one_below() {
    assert_eq!(sample_count(8, &SUPPORTED), 4);
    assert_eq!(sample_count(3, &SUPPORTED), 2);
}

#[test]
fn zero_disables_msaa() {
    assert_eq!(sample_count(0, &SUPPORTED), 1);
}
