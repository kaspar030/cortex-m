//! Inline assembly implementing the routines exposed in `cortex_m::asm`.
//!
//! If the `inline-asm` feature is enabled, these functions will be directly called by the
//! `cortex-m` wrappers. Otherwise, `cortex-m` links against them via prebuilt archives.
//!
//! All of these functions should be blanket-`unsafe`. `cortex-m` provides safe wrappers where
//! applicable.

use core::sync::atomic::{Ordering, compiler_fence};

#[inline(always)]
pub unsafe fn __bkpt() {
    asm!("bkpt");
}

#[inline(always)]
pub unsafe fn __control_r() -> u32 {
    let r;
    asm!("mrs {}, CONTROL", out(reg) r);
    r
}

#[inline(always)]
pub unsafe fn __control_w(w: u32) {
    // ISB is required after writing to CONTROL,
    // per ARM architectural requirements (see Application Note 321).
    asm!(
        "msr CONTROL, {}",
        "isb",
        in(reg) w
    );

    // Ensure memory accesses are not reordered around the CONTROL update.
    compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn __cpsid() {
    asm!("cpsid i");

    // Ensure no subsequent memory accesses are reordered to before interrupts are disabled.
    compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn __cpsie() {
    // Ensure no preceeding memory accesses are reordered to after interrupts are enabled.
    compiler_fence(Ordering::SeqCst);

    asm!("cpsie i");
}

#[inline(always)]
pub unsafe fn __delay(cyc: u32) {
    // Use local labels to avoid R_ARM_THM_JUMP8 relocations which fail on thumbv6m.
    asm!(
        "1:",
        "nop",
        "subs {}, #1",
        "bne 1b",
        in(reg) cyc
    );
}

#[inline(always)]
pub unsafe fn __dmb() {
    asm!("dmb");
    compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn __dsb() {
    asm!("dsb");
    compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn __isb() {
    asm!("isb");
    compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn __msp_r() -> u32 {
    let r;
    asm!("mrs {}, MSP", out(reg) r);
    r
}

#[inline(always)]
pub unsafe fn __msp_w(val: u32) {
    asm!("msr MSP, {}", in(reg) val);
}

// NOTE: No FFI shim, this requires inline asm.
#[inline(always)]
pub unsafe fn __apsr_r() -> u32 {
    let r;
    asm!("mrs {}, APSR", out(reg) r);
    r
}

#[inline(always)]
pub unsafe fn __nop() {
    // NOTE: This is a `pure` asm block, but applying that option allows the compiler to eliminate
    // the nop entirely (or to collapse multiple subsequent ones). Since the user probably wants N
    // nops when they call `nop` N times, let's not add that option.
    asm!("nop");
}

// NOTE: No FFI shim, this requires inline asm.
#[inline(always)]
pub unsafe fn __pc_r() -> u32 {
    let r;
    asm!("mov {}, pc", out(reg) r);
    r
}

// NOTE: No FFI shim, this requires inline asm.
#[inline(always)]
pub unsafe fn __pc_w(val: u32) {
    asm!("mov pc, {}", in(reg) val);
}

// NOTE: No FFI shim, this requires inline asm.
#[inline(always)]
pub unsafe fn __lr_r() -> u32 {
    let r;
    asm!("mov {}, lr", out(reg) r);
    r
}

// NOTE: No FFI shim, this requires inline asm.
#[inline(always)]
pub unsafe fn __lr_w(val: u32) {
    asm!("mov lr, {}", in(reg) val);
}

#[inline(always)]
pub unsafe fn __primask_r() -> u32 {
    let r;
    asm!("mrs {}, PRIMASK", out(reg) r);
    r
}

#[inline(always)]
pub unsafe fn __psp_r() -> u32 {
    let r;
    asm!("mrs {}, PSP", out(reg) r);
    r
}

#[inline(always)]
pub unsafe fn __psp_w(val: u32) {
    asm!("msr PSP, {}", in(reg) val);
}

#[inline(always)]
pub unsafe fn __sev() {
    asm!("sev");
}

#[inline(always)]
pub unsafe fn __udf() -> ! {
    asm!("udf #0", options(noreturn));
}

#[inline(always)]
pub unsafe fn __wfe() {
    asm!("wfe");
}

#[inline(always)]
pub unsafe fn __wfi() {
    asm!("wfi");
}

// v7m *AND* v8m.main, but *NOT* v8m.base
#[cfg(any(armv7m, armv8m_main))]
pub use self::v7m::*;
#[cfg(any(armv7m, armv8m_main))]
mod v7m {
    use core::sync::atomic::{Ordering, compiler_fence};

    #[inline(always)]
    pub unsafe fn __basepri_max(val: u8) {
        asm!("msr BASEPRI_MAX, {}", in(reg) val);
    }

    #[inline(always)]
    pub unsafe fn __basepri_r() -> u8 {
        let r;
        asm!("mrs {}, BASEPRI", out(reg) r);
        r
    }

    #[inline(always)]
    pub unsafe fn __basepri_w(val: u8) {
        asm!("msr BASEPRI, {}", in(reg) val);
    }

    #[inline(always)]
    pub unsafe fn __faultmask_r() -> u32 {
        let r;
        asm!("mrs {}, FAULTMASK", out(reg) r);
        r
    }

    #[inline(always)]
    pub unsafe fn __enable_icache() {
        asm!(
            "ldr {0}, =0xE000ED14",         // CCR
            "mrs {2}, PRIMASK",             // save critical nesting info
            "cpsid i",                      // mask interrupts
            "ldr {1}, [{0}]",               // read CCR
            "orr.w {1}, {1}, #(1 << 17)",   // Set bit 17, IC
            "str {1}, [{0}]",               // write it back
            "dsb",                          // ensure store completes
            "isb",                          // synchronize pipeline
            "msr PRIMASK, {2}",             // unnest critical section
            out(reg) _,
            out(reg) _,
            out(reg) _,
        );
        compiler_fence(Ordering::SeqCst);
    }

    #[inline(always)]
    pub unsafe fn __enable_dcache() {
        asm!(
            "ldr {0}, =0xE000ED14",         // CCR
            "mrs {2}, PRIMASK",             // save critical nesting info
            "cpsid i",                      // mask interrupts
            "ldr {1}, [{0}]",               // read CCR
            "orr.w {1}, {1}, #(1 << 16)",   // Set bit 16, DC
            "str {1}, [{0}]",               // write it back
            "dsb",                          // ensure store completes
            "isb",                          // synchronize pipeline
            "msr PRIMASK, {2}",             // unnest critical section
            out(reg) _,
            out(reg) _,
            out(reg) _,
        );
        compiler_fence(Ordering::SeqCst);
    }
}

#[cfg(armv7em)]
pub use self::v7em::*;
#[cfg(armv7em)]
mod v7em {
    #[inline(always)]
    pub unsafe fn __basepri_max_cm7_r0p1(val: u8) {
        asm!(
            "mrs {1}, PRIMASK",
            "cpsid i",
            "tst.w {1}, #1",
            "msr BASEPRI_MAX, {0}",
            "it ne",
            "bxne lr",
            "cpsie i",
            in(reg) val,
            out(reg) _,
        );
    }

    #[inline(always)]
    pub unsafe fn __basepri_w_cm7_r0p1(val: u8) {
        asm!(
            "mrs {1}, PRIMASK",
            "cpsid i",
            "tst.w {1}, #1",
            "msr BASEPRI, {0}",
            "it ne",
            "bxne lr",
            "cpsie i",
            in(reg) val,
            out(reg) _,
        );
    }
}

#[cfg(armv8m)]
pub use self::v8m::*;
/// Baseline and Mainline.
#[cfg(armv8m)]
mod v8m {
    #[inline(always)]
    pub unsafe fn __tt(mut target: u32) -> u32 {
        asm!("tt {target}, {target}", target = inout(reg) target);
        target
    }

    #[inline(always)]
    pub unsafe fn __ttt(mut target: u32) -> u32 {
        asm!("ttt {target}, {target}", target = inout(reg) target);
        target
    }

    #[inline(always)]
    pub unsafe fn __tta(mut target: u32) -> u32 {
        asm!("tta {target}, {target}", target = inout(reg) target);
        target
    }

    #[inline(always)]
    pub unsafe fn __ttat(mut target: u32) -> u32 {
        asm!("ttat {target}, {target}", target = inout(reg) target);
        target
    }
}

#[cfg(armv8m_main)]
pub use self::v8m_main::*;
/// Mainline only.
#[cfg(armv8m_main)]
mod v8m_main {
    #[inline(always)]
    pub unsafe fn __msplim_r() -> u32 {
        let r;
        asm!("mrs {}, MSPLIM", out(reg) r);
        r
    }

    #[inline(always)]
    pub unsafe fn __msplim_w(val: u32) {
        asm!("msr MSPLIM, {}", in(reg) val);
    }

    #[inline(always)]
    pub unsafe fn __psplim_r() -> u32 {
        let r;
        asm!("mrs {}, PSPLIM", out(reg) r);
        r
    }

    #[inline(always)]
    pub unsafe fn __psplim_w(val: u32) {
        asm!("msr PSPLIM, {}", in(reg) val);
    }
}

#[cfg(has_fpu)]
pub use self::fpu::*;
/// All targets with FPU.
#[cfg(has_fpu)]
mod fpu {
    #[inline(always)]
    pub unsafe fn __fpscr_r() -> u32 {
        let r;
        asm!("vmrs {}, fpscr", out(reg) r);
        r
    }

    #[inline(always)]
    pub unsafe fn __fpscr_w(val: u32) {
        asm!("vmsr fpscr, {}", in(reg) val);
    }
}
