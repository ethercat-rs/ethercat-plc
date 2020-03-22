// Part of ethercat-rs. Copyright 2018-2019 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Tools to create a typesafe process image matching with possible slave PDOs.

use ethercat::*;

pub trait ProcessImage {
    // configuration APIs
    const SLAVE_COUNT: usize;
    fn get_slave_ids() -> Vec<SlaveId>;
    fn get_slave_pdos() -> Vec<Option<Vec<SyncInfo<'static>>>> { vec![None] }
    fn get_slave_regs() -> Vec<Vec<(PdoEntryIndex, Offset)>> { vec![vec![]] }
    fn get_slave_sdos<C: ProcessConfig>(_: &C) -> Vec<Vec<(SdoIndex, &dyn SdoData)>> { vec![vec![]] }

    fn size() -> usize where Self: Sized {
        std::mem::size_of::<Self>()
    }

    fn cast(data: &mut [u8]) -> &mut Self where Self: Sized {
        unsafe { std::mem::transmute(data.as_mut_ptr()) }
    }
}

pub trait ExternImage : Default {
    fn size() -> usize where Self: Sized {
        std::mem::size_of::<Self>()
    }

    fn cast(&mut self) -> &mut [u8] where Self: Sized {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut _ as *mut u8, Self::size())
        }
    }
}

pub trait ProcessConfig {
    fn get_sdo_var(&self, var: &str) -> Option<&dyn SdoData>;
}

impl ProcessConfig for () {
    fn get_sdo_var(&self, _: &str) -> Option<&dyn SdoData> {
        None
    }
}

impl ProcessConfig for std::collections::HashMap<String, Box<dyn SdoData>> {
    fn get_sdo_var(&self, var: &str) -> Option<&dyn SdoData> {
        self.get(var).map(|s| &**s)
    }
}

// TODO: add a derive macro for ProcessConfig so that you can configure
// the PLC using a well typed struct
