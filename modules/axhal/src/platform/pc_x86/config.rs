use crate::mem::MemRegionFlags;
use bitflags::bitflags;
use core::fmt::Debug;
use core::{mem::size_of, slice};
use page_table_entry::MappingFlags;

const _CONFIG_SIGNATURE: [u8; 5] = *b"ACSYS";
const _CONFIG_REVISION: u16 = 14;
pub const PER_CPU_ARRAY_PTR: *mut HvSystemConfig = __core_end as _;

const HV_CELL_NAME_MAXLEN: usize = 31;
const HV_MAX_IOMMU_UNITS: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemFlags(u64);

bitflags! {
    impl MemFlags: u64 {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DMA           = 1 << 3;
        const IO            = 1 << 4;
        const COMM_REGION   = 1 << 5;
        const LOADABLE      = 1 << 6;
        const ROOTSHARED    = 1 << 7;
        const NO_HUGEPAGES  = 1 << 8;
        const USER          = 1 << 9;
    }
}

impl From<MemFlags> for MemRegionFlags {
    fn from(f: MemFlags) -> Self {
        let mut ret = MemRegionFlags::empty();
        if f.contains(MemFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemFlags::IO) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemFlags::DMA) {
            ret |= Self::DMA;
        }
        if f.contains(MemFlags::USER) {
            ret |= Self::USER;
        }
        ret
    }
}

impl From<MemFlags> for MappingFlags {
    fn from(f: MemFlags) -> Self {
        let mut ret = MappingFlags::empty();
        if f.contains(MemFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemFlags::IO) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemFlags::USER) {
            ret |= Self::USER;
        }
        ret
    }
}

extern "C" {
    fn __core_end();
}

/// Matches C `struct jailhouse_console` in `jailhouse/console.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvConsole {
    pub address: u64,
    pub size: u32,
    pub console_type: u16,
    pub flags: u16,
    pub divider: u32,
    pub gate_nr: u32,
    pub clock_reg: u64,
}

/// Matches C `struct jailhouse_cell_desc` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvCellDesc {
    pub signature: [u8; 5],
    pub architecture: u8,
    pub revision: u16,

    pub name: [u8; HV_CELL_NAME_MAXLEN + 1],
    pub id: u32, // set by the driver
    pub flags: u32,

    pub cpu_set_size: u32,
    pub num_memory_regions: u32,
    pub num_cache_regions: u32,
    pub num_irqchips: u32,
    pub num_pio_regions: u32,
    pub num_pci_devices: u32,
    pub num_pci_caps: u32,
    pub num_stream_ids: u32,

    pub vpci_irq_base: u32,

    pub cpu_reset_address: u64,
    pub msg_reply_timeout: u64,

    pub console: HvConsole,
}

/// Matches C `struct jailhouse_memory` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvMemoryRegion {
    pub phys_start: u64,
    pub virt_start: u64,
    pub size: u64,
    pub flags: MemFlags,
}

/// Matches C `struct jailhouse_cache` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvCacheRegion {
    pub start: u32,
    pub size: u32,
    pub cache_type: u8,
    pub padding: u8,
    pub flags: u16,
}

/// Matches C `struct jailhouse_irqchip` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvIrqChip {
    pub address: u64,
    pub id: u32,
    pub pin_base: u32,
    pub pin_bitmap: [u32; 4],
}

/// Matches C `struct jailhouse_pio` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvPioRegion {
    pub base: u16,
    pub length: u16,
}

/// Matches C `struct jailhouse_pci_device` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvPciDevice {
    pub pci_device_type: u8,
    pub iommu: u8,
    pub domain: u16,
    pub bdf: u16,
    pub bar_mask: [u32; 6],
    pub caps_start: u16,
    pub num_caps: u16,
    pub num_msi_vectors: u8,
    pub msi_flags: u8,
    pub num_msix_vectors: u16,
    pub msix_region_size: u16,
    pub msix_address: u64,
    pub shmem_regions_start: u32,
    pub shmem_dev_id: u8,
    pub shmem_peers: u8,
    pub shmem_protocol: u16,
}

/// Matches C `struct jailhouse_pci_capability` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvPciCapability {
    pub id: u16,
    pub start: u16,
    pub len: u16,
    pub flags: u16,
}

/// Matches C `struct jailhouse_iommu` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvIommu {
    pub iommu_type: u32,
    pub base: u64,
    pub size: u32,
    pub amd_bdf: u16,
    pub amd_base_cap: u8,
    pub amd_msi_cap: u8,
    pub amd_features: u32,
    pub _padding: [u8; 4],
}

/// Matches C `union jailhouse_stream_id` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvStreamId {
    pub id: u32,
}

/// Matches x86 arch struct in C `struct jailhouse_system` platform_info in `jailhouse/cell-config.h`
#[cfg(target_arch = "x86_64")]
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ArchPlatformInfo {
    pub pm_timer_address: u16,
    pub apic_mode: u8,
    pub padding: u8,
    pub vtd_interrupt_limit: u32,
    pub tsc_khz: u32,
    pub apic_khz: u32,
    pub _padding: [u8; 28],
}

/// Matches C `platform_info` in `struct jailhouse_system` in `jailhouse/cell-config.h`
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct PlatformInfo {
    pub pci_mmconfig_base: u64,
    pub pci_mmconfig_end_bus: u8,
    pub pci_is_virtual: u8,
    pub pci_domain: u16,
    pub iommu_units: [HvIommu; HV_MAX_IOMMU_UNITS],
    pub arch: ArchPlatformInfo,
}

/// General descriptor of the system.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct HvSystemConfig {
    pub signature: [u8; 5],
    pub architecture: u8,
    pub revision: u16,
    pub flags: u32,

    pub hypervisor_memory: HvMemoryRegion,
    pub debug_console: HvConsole,
    pub platform_info: PlatformInfo,
    pub root_cell: HvCellDesc,
}

pub struct CellConfig<'a> {
    desc: &'a HvCellDesc,
}

impl HvCellDesc {
    pub const fn config(&self) -> CellConfig {
        CellConfig::from(self)
    }

    pub const fn config_size(&self) -> usize {
        self.cpu_set_size as usize
            + self.num_memory_regions as usize * size_of::<HvMemoryRegion>()
            + self.num_cache_regions as usize * size_of::<HvCacheRegion>()
            + self.num_irqchips as usize * size_of::<HvIrqChip>()
            + self.num_pio_regions as usize * size_of::<HvPioRegion>()
            + self.num_pci_devices as usize * size_of::<HvPciDevice>()
            + self.num_pci_caps as usize * size_of::<HvPciCapability>()
            + self.num_stream_ids as usize * size_of::<HvStreamId>()
    }
}

impl HvSystemConfig {
    pub fn get<'a>() -> &'a Self {
        unsafe { &*super::consts::hv_config_ptr() }
    }

    pub const fn size(&self) -> usize {
        size_of::<Self>() + self.root_cell.config_size()
    }
}

impl<'a> CellConfig<'a> {
    const fn from(desc: &'a HvCellDesc) -> Self {
        Self { desc }
    }

    fn config_ptr<T>(&self) -> *const T {
        unsafe { (self.desc as *const HvCellDesc).add(1) as _ }
    }

    pub const fn size(&self) -> usize {
        self.desc.config_size()
    }

    pub fn cpu_set(&self) -> &[u64] {
        unsafe { slice::from_raw_parts(self.config_ptr(), self.desc.cpu_set_size as usize / 8) }
    }

    pub fn mem_regions(&self) -> &[HvMemoryRegion] {
        unsafe {
            let ptr = self.cpu_set().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_memory_regions as usize)
        }
    }

    pub fn cache_regions(&self) -> &[HvCacheRegion] {
        unsafe {
            let ptr = self.mem_regions().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_cache_regions as usize)
        }
    }

    pub fn irqchips(&self) -> &[HvIrqChip] {
        unsafe {
            let ptr = self.cache_regions().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_irqchips as usize)
        }
    }

    pub fn pio_regions(&self) -> &[HvPioRegion] {
        unsafe {
            let ptr = self.irqchips().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_pio_regions as usize)
        }
    }

    pub fn pci_devices(&self) -> &[HvPciDevice] {
        unsafe {
            let ptr = self.pio_regions().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_pci_devices as usize)
        }
    }

    pub fn pci_caps(&self) -> &[HvPciCapability] {
        unsafe {
            let ptr = self.pci_devices().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_pci_caps as usize)
        }
    }

    pub fn stream_ids(&self) -> &[HvStreamId] {
        unsafe {
            let ptr = self.pci_caps().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_stream_ids as usize)
        }
    }
}


// impl Debug for CellConfig<'_> {
//     fn fmt(&self, f: &mut Formatter) -> Result {
//         let name = self.desc.name;
//         let mut len = 0;
//         while name[len] != 0 {
//             len += 1;
//         }
//         f.debug_struct("CellConfig")
//             .field("name", &core::str::from_utf8(&name[..len]))
//             .field("size", &self.size())
//             .field("mem_regions", &self.mem_regions())
//             .finish()
//     }
// }
