#!/usr/bin/env nu
if ("_out" | path exists) { rm --recursive _out }
mkdir _out
cargo check

$env.RUST_BACKTRACE = 1;

round_trip "listing_0037_single_register_mov"
round_trip "listing_0038_many_register_mov"
round_trip "listing_0039_more_movs"
round_trip "listing_0040_challenge_movs"
round_trip "listing_0041_add_sub_cmp_jnz"

compare_stdout "listing_0043_immediate_movs" false
compare_stdout "listing_0044_register_movs" false
compare_stdout "listing_0045_challenge_register_movs" false
compare_stdout "listing_0046_add_sub_cmp" false
# compare_stdout "listing_0047_challenge_flags" false

compare_stdout "listing_0048_ip_register" true


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

def compare_stdout [case, print_ip?] {
    let listing_dir = "../computer_enhance/perfaware/part1"
    print $"Test\(compare_stdout\): ($case)"
    mut args = ["--quiet", "--", $"($listing_dir)/($case)"]
    if $print_ip {
        $args = $args | append "--print-ip"
    }
    cargo run ...$args | save $"_out/($case).actual.txt"
    difft --exit-code $"($listing_dir)/($case).txt" $"./_out/($case).actual.txt"
    print $"OK: ($case)"
}
