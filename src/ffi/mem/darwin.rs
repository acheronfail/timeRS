#![allow(non_camel_case_types)]

use anyhow::{bail, Result};
use libc::{integer_t, kern_return_t, mach_msg_type_number_t, mach_port_t, natural_t};
use std::mem::{size_of, MaybeUninit};

type host_t = mach_port_t;
type host_flavor_t = integer_t;

extern "C" {
    fn host_statistics64(
        host_priv: host_t,
        flavor: host_flavor_t,
        host_info64_out: *mut vm_statistics64,
        host_info64_outCnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Hash, PartialOrd, PartialEq, Eq, Ord)]
pub struct vm_statistics64 {
    pub free_count: natural_t,      /* # of pages free */
    pub active_count: natural_t,    /* # of pages active */
    pub inactive_count: natural_t,  /* # of pages inactive */
    pub wire_count: natural_t,      /* # of pages wired down */
    pub zero_fill_count: u64,       /* # of zero fill pages */
    pub reactivations: u64,         /* # of pages reactivated */
    pub pageins: u64,               /* # of pageins */
    pub pageouts: u64,              /* # of pageouts */
    pub faults: u64,                /* # of faults */
    pub cow_faults: u64,            /* # of copy-on-writes */
    pub lookups: u64,               /* object cache lookups */
    pub hits: u64,                  /* object cache hits */
    pub purges: u64,                /* # of pages purged */
    pub purgeable_count: natural_t, /* # of pages purgeable */
    /*
     * NB: speculative pages are already accounted for in "free_count",
     * so "speculative_count" is the number of "free" pages that are
     * used to hold data that was read speculatively from disk but
     * haven't actually been used by anyone so far.
     */
    pub speculative_count: natural_t, /* # of pages speculative */

    /* added for rev1 */
    pub decompressions: u64,              /* # of pages decompressed */
    pub compressions: u64,                /* # of pages compressed */
    pub swapins: u64,                     /* # of pages swapped in (via compression segments) */
    pub swapouts: u64,                    /* # of pages swapped out (via compression segments) */
    pub compressor_page_count: natural_t, /* # of pages used by the compressed pager to hold all the compressed data */
    pub throttled_count: natural_t,       /* # of pages throttled */
    pub external_page_count: natural_t,   /* # of pages that are file-backed (non-swap) */
    pub internal_page_count: natural_t,   /* # of pages that are anonymous */
    pub total_uncompressed_pages_in_compressor: u64, /* # of pages (uncompressed) held within the compressor. */
}

pub const HOST_VM_INFO64_COUNT: usize = size_of::<vm_statistics64>() / size_of::<integer_t>();
pub const HOST_VM_INFO64: usize = 4;

pub fn memory_available() -> Result<u64> {
    let mut stats: MaybeUninit<vm_statistics64> = MaybeUninit::uninit();
    let mut count = HOST_VM_INFO64_COUNT as u32;

    // TODO: doesn't take into account stolen pages, and need to double check calculations
    // https://stackoverflow.com/a/43300124/5552584
    let ret = unsafe {
        host_statistics64(
            libc::mach_host_self(),
            HOST_VM_INFO64 as i32,
            stats.as_mut_ptr(),
            (&mut count) as *mut mach_msg_type_number_t,
        )
    };

    if ret != libc::KERN_SUCCESS {
        bail!("Call to host_statistics64 returned non-zero result: {}", ret);
    }

    // SAFETY: we have asserted that the return code was successful
    let stats = unsafe { stats.assume_init() };
    log::trace!("{:#?}", stats);
    return Ok(
        (stats.external_page_count + stats.purgeable_count + stats.free_count
            - stats.speculative_count) as u64
            * page_size()?,
    );
}
