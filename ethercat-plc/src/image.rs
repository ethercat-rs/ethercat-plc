// Part of ethercat-rs. Copyright 2018-2024 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Tools to create a typesafe process image matching with possible slave PDOs.

use ethercat::*;

pub trait ProcessImage {
    // configuration APIs
    const SLAVE_COUNT: usize;
    fn get_slave_ids() -> Vec<SlaveId>;
    fn get_slave_pdos() -> Vec<Option<Vec<(SmCfg, Vec<PdoCfg>)>>> { vec![None] }
    fn get_slave_regs() -> Vec<Vec<(PdoEntryIdx, Offset)>> { vec![vec![]] }
    fn get_slave_sdos<C: ProcessConfig>(_: &C) -> Vec<Vec<(SdoIdx, &dyn SdoData)>> { vec![vec![]] }
    fn get_slave_wd_dc() -> Vec<(Option<(u16, u16)>, Option<(u16, u32, i32, u32, i32)>)> {
        vec![(None, None)]
    }

    fn size() -> usize where Self: Sized {
        std::mem::size_of::<Self>()
    }

    fn cast(data: &mut [u8]) -> &mut Self where Self: Sized {
        unsafe { &mut *data.as_mut_ptr().cast() }
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

impl<'a> ProcessConfig for std::collections::HashMap<&'a str, Box<dyn SdoData>> {
    fn get_sdo_var(&self, var: &str) -> Option<&dyn SdoData> {
        self.get(var).map(|s| &**s)
    }
}

// TODO: add a derive macro for ProcessConfig so that you can configure
// the PLC using a well typed struct
