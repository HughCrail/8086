#!/usr/bin/env nu
if ("_out" | path exists) { rm --recursive _out }
mkdir _out
cargo check

round_trip "37-small-asm-ex"
round_trip "38-larger-asm-ex"
round_trip "39"
round_trip "40"
round_trip "41"

def round_trip [case] {
    print $"Test: ($case)"
    nasm $"../listings/($case).asm"
    xxd $"../listings/($case)" | save --force $"../listings/($case).hex"
    cargo run --quiet -- $"../listings/($case)" -o $"_out/($case).asm"
    nasm $"_out/($case).asm"
    xxd $"_out/($case)" | save $"_out/($case).hex"
    diff $"_out/($case).hex" $"../listings/($case).hex"
    print $"OK: ($case)"
}

