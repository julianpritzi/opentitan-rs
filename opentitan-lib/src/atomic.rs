#[cfg(all(not(feature = "atomic_emulation"), any(target_has_atomic)))]
compile_error!(
    "Compiling for target that supports atomics, which opentitan does not.
If atomics are required try to turn on the 'atomic_emulation' feature of \
opentitan-lib, else change the target (riscv32imac -> riscv32imc) "
);

#[cfg(feature = "atomic_emulation")]
mod emulation {
    #[cfg(all(not(feature = "silent_atomic_emulation"), not(target_has_atomic)))]
    compile_error!(
        "Atomic emulation is turned on but compilation target does not support atomics.
This is likely an error, if you wish to use the core libraries atomics, \
change the target (riscv32imc -> riscv32imac), else if this is \
intended use the 'silent_atomic_emulation' feature"
    );
}
