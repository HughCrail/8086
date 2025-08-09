#!/usr/bin/env nu
if ("_out" | path exists) { rm --recursive _out }
mkdir _out
cargo check

round_trip "listing_0037_single_register_mov"
round_trip "listing_0038_many_register_mov"
round_trip "listing_0039_more_movs"
round_trip "listing_0040_challenge_movs"
round_trip "listing_0041_add_sub_cmp_jnz"

compare_stdout "listing_0043_immediate_movs"
compare_stdout "listing_0044_register_movs"

def round_trip [case] {
    let listing_dir = "../computer_enhance/perfaware/part1"
    print $"Test\(round_trip\): ($case)"
    xxd $"($listing_dir)/($case)" | save --force $"./_out/($case).expected.hex"
    cargo run --quiet -- $"($listing_dir)/($case)" -o $"_out/($case).asm" e> /dev/null
    nasm $"_out/($case).asm"
    xxd $"_out/($case)" | save $"_out/($case).actual.hex"
    difft --exit-code $"./_out/($case).expected.hex" $"_out/($case).actual.hex" 
    print $"OK: ($case)"
}

def compare_stdout [case] {
    let listing_dir = "../computer_enhance/perfaware/part1"
    print $"Test\(compare_stdout\): ($case)"
    cargo run --quiet -- $"($listing_dir)/($case)" | save $"_out/($case).actual.txt"
    difft --exit-code $"./_out/($case).actual.txt" $"($listing_dir)/($case).txt" 
    print $"OK: ($case)"
}
