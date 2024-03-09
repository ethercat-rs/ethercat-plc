// Part of ethercat-rs. Copyright 2018-2024 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

use ethercat_derive::SlaveProcessImage;
use crate::image::ProcessImage;

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EK1100 {}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EK1110 {}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EK1818 {
    #[entry(0x6000, 1)]  pub input: u8,
    #[entry(0x7000, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL1008 {
    #[entry(0x6000, 1)]  pub input: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL1018 {
    #[entry(0x6000, 1)]  pub input: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A00, 0x1A01)]
#[pdos(2, Output, 0x1600, 0x1601)]
pub struct EL1502 {
    #[entry(0x1A00, 0x6000, 1)]  pub status_ch1: u16,
    #[entry(0x1A00, 0x6000, 17)] pub value_ch1: u32,
    #[entry(0x1A01, 0x6010, 1)]  pub status_ch2: u16,
    #[entry(0x1A01, 0x6010, 17)] pub value_ch2: u32,

    #[entry(0x1600, 0x7000, 1)]  pub control_ch1: u16,
    #[entry(0x1600, 0x7000, 17)] pub setvalue_ch1: u32,
    #[entry(0x1601, 0x7010, 1)]  pub control_ch2: u16,
    #[entry(0x1601, 0x7010, 17)] pub setvalue_ch2: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A02)]
#[pdos(2, Output, 0x1602)]
#[allow(non_camel_case_types)]
pub struct EL1502_UpDown {
    #[entry(0x1A02, 0x6020, 1)]  pub status: u16,
    #[entry(0x1A02, 0x6020, 17)] pub value: u32,

    #[entry(0x1602, 0x7020, 1)]  pub control: u16,
    #[entry(0x1602, 0x7020, 17)] pub setvalue: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL1859 {
    #[entry(0x6000, 1)]  pub input: u8,
    #[entry(0x7080, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL2008 {
    #[entry(0x7000, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A00, 0x1A02)]
#[pdos(2, Output, 0x1600, 0x1601)]
pub struct EL2535 {
    #[entry(0x1A00, 0x6000, 1)]    pub state_ch1: u16,
    #[entry(0x1A02, 0x6010, 1)]    pub state_ch2: u16,

    #[entry(0x1600, 0x7000, 1)]    pub control_ch1: u16,
    #[entry(0x1600, 0x7000, 0x11)] pub output_ch1: u16,
    #[entry(0x1601, 0x7010, 1)]    pub control_ch2: u16,
    #[entry(0x1601, 0x7010, 0x11)] pub output_ch2: u16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(2, Output, 0x1600, 0x1610, 0x1620, 0x1630)]
#[pdos(3, Input,  0x1A00, 0x1A10, 0x1A20, 0x1A30)]
pub struct EL2574 {
    #[entry(0x1A00, 0x6000, 1)]  pub ch1_status: u16,
    #[entry(0x1A10, 0x6010, 1)]  pub ch2_status: u16,
    #[entry(0x1A20, 0x6020, 1)]  pub ch3_status: u16,
    #[entry(0x1A30, 0x6030, 1)]  pub ch4_status: u16,

    #[entry(0x1600, 0x7000, 1)]    pub ch1_control: u8,
    #[entry(0x1600, 0x7000, 9)]    pub ch1_command: u16,
    #[entry(0x1600, 0x7000, 0x11)] pub ch1_index: u16,
    #[entry(0x1600, 0x7000, 0x12)] pub ch1_length: u16,
    #[entry(0x1600, 0x7000, 0x13)] pub ch1_param: u8,
    #[entry(0x1600, 0x7000, 0x21)] pub ch1_red: u8,
    #[entry(0x1600, 0x7000, 0x22)] pub ch1_green: u8,
    #[entry(0x1600, 0x7000, 0x23)] pub ch1_blue: u8,
    #[entry(0x1600, 0x7000, 0x24)] pub ch1_white: u8,

    #[entry(0x1600, 0x7010, 1)]    pub ch2_control: u8,
    #[entry(0x1600, 0x7010, 9)]    pub ch2_command: u16,
    #[entry(0x1600, 0x7010, 0x11)] pub ch2_index: u16,
    #[entry(0x1600, 0x7010, 0x12)] pub ch2_length: u16,
    #[entry(0x1600, 0x7010, 0x13)] pub ch2_param: u8,
    #[entry(0x1600, 0x7010, 0x21)] pub ch2_red: u8,
    #[entry(0x1600, 0x7010, 0x22)] pub ch2_green: u8,
    #[entry(0x1600, 0x7010, 0x23)] pub ch2_blue: u8,
    #[entry(0x1600, 0x7010, 0x24)] pub ch2_white: u8,

    #[entry(0x1600, 0x7020, 1)]    pub ch3_control: u8,
    #[entry(0x1600, 0x7020, 9)]    pub ch3_command: u16,
    #[entry(0x1600, 0x7020, 0x11)] pub ch3_index: u16,
    #[entry(0x1600, 0x7020, 0x12)] pub ch3_length: u16,
    #[entry(0x1600, 0x7020, 0x13)] pub ch3_param: u8,
    #[entry(0x1600, 0x7020, 0x21)] pub ch3_red: u8,
    #[entry(0x1600, 0x7020, 0x22)] pub ch3_green: u8,
    #[entry(0x1600, 0x7020, 0x23)] pub ch3_blue: u8,
    #[entry(0x1600, 0x7020, 0x24)] pub ch3_white: u8,

    #[entry(0x1600, 0x7030, 1)]    pub ch4_control: u8,
    #[entry(0x1600, 0x7030, 9)]    pub ch4_command: u16,
    #[entry(0x1600, 0x7030, 0x11)] pub ch4_index: u16,
    #[entry(0x1600, 0x7030, 0x12)] pub ch4_length: u16,
    #[entry(0x1600, 0x7030, 0x13)] pub ch4_param: u8,
    #[entry(0x1600, 0x7030, 0x21)] pub ch4_red: u8,
    #[entry(0x1600, 0x7030, 0x22)] pub ch4_green: u8,
    #[entry(0x1600, 0x7030, 0x23)] pub ch4_blue: u8,
    #[entry(0x1600, 0x7030, 0x24)] pub ch4_white: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(2, Output, 0x1601)]
#[pdos(3, Input,  0x1A00)]
pub struct EL2574_Extended {
    #[entry(0x1A00, 0x6000, 1)]  pub ch1_status: u16,
    // #[entry(0x1A10, 0x6010, 1)]  pub ch2_status: u16,
    // #[entry(0x1A20, 0x6020, 1)]  pub ch3_status: u16,
    // #[entry(0x1A30, 0x6030, 1)]  pub ch4_status: u16,

    #[entry(0x1601, 0x7001, 1)]    pub ch1_control: u8,
    #[entry(0x1601, 0x7001, 9)]    pub ch1_index: u8,
    #[entry(0x1601, 0x7001, 0x11)] pub ch1_el0: u32,
    #[entry(0x1601, 0x7001, 0x12)] pub ch1_el1: u32,
    #[entry(0x1601, 0x7001, 0x13)] pub ch1_el2: u32,
    #[entry(0x1601, 0x7001, 0x14)] pub ch1_el3: u32,
    #[entry(0x1601, 0x7001, 0x15)] pub ch1_el4: u32,
    #[entry(0x1601, 0x7001, 0x16)] pub ch1_el5: u32,
    #[entry(0x1601, 0x7001, 0x17)] pub ch1_el6: u32,
    #[entry(0x1601, 0x7001, 0x18)] pub ch1_el7: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL2612 {
    #[entry(0x7000, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL2622 {
    #[entry(0x7000, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL2624 {
    #[entry(0x7000, 1)]  pub output: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL3104 {
    #[entry(0x6000, 1)]  pub ch1_status: u16,
    #[entry(0x6000, 17)] pub ch1: i16,
    #[entry(0x6010, 1)]  pub ch2_status: u16,
    #[entry(0x6010, 17)] pub ch2: i16,
    #[entry(0x6020, 1)]  pub ch3_status: u16,
    #[entry(0x6020, 17)] pub ch3: i16,
    #[entry(0x6030, 1)]  pub ch4_status: u16,
    #[entry(0x6030, 17)] pub ch4: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A02, 0x1A04)]
pub struct EL3152 {
    #[entry(0x1A02, 0x6000, 1)]  pub ch1_status: u16,
    #[entry(0x1A02, 0x6000, 17)] pub ch1: i16,
    #[entry(0x1A04, 0x6010, 1)]  pub ch2_status: u16,
    #[entry(0x1A04, 0x6010, 17)] pub ch2: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL4132 {
    #[entry(0x3001, 1)]  pub ch1: i16,
    #[entry(0x3002, 1)]  pub ch2: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL5001 {
    #[entry(0x3101, 1)] pub status_ch1: u8,
    #[entry(0x3101, 2)] pub value_ch1: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input, 0x1A00, 0x1A01)]
pub struct EL5002 {
    #[entry(0x1A00, 0x6000, 1)]  pub status_ch1: u16,
    #[entry(0x1A00, 0x6000, 11)] pub value_ch1: u32,
    #[entry(0x1A01, 0x6010, 1)]  pub status_ch2: u16,
    #[entry(0x1A01, 0x6010, 11)] pub value_ch2: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input, 0x1A00, 0x1A01)]
#[watchdog(1, 1)]
pub struct EL5032 {
    #[entry(0x1A00, 0x6000, 1)]  pub status_ch1: u16,
    #[entry(0x1A00, 0x6000, 11)] pub value_ch1: u64,
    #[entry(0x1A01, 0x6010, 1)]  pub status_ch2: u16,
    #[entry(0x1A01, 0x6010, 11)] pub value_ch2: u64,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A00, 0x1A02)]
#[pdos(2, Output, 0x1600, 0x1601)]
pub struct EL5072 {
    #[entry(0x1A00, 0x6000, 1)]  pub status_ch1: u16,
    #[entry(0x1A00, 0x6001, 1)]  pub value_ch1: i32,
    #[entry(0x1A00, 0x6001, 2)]  pub latch_ch1: i32,
    #[entry(0x1A02, 0x6010, 1)]  pub status_ch2: u16,
    #[entry(0x1A02, 0x6011, 1)]  pub value_ch2: i32,
    #[entry(0x1A02, 0x6011, 2)]  pub latch_ch2: i32,

    #[entry(0x1600, 0x7000, 1)]    pub control_ch1: u32,
    #[entry(0x1600, 0x7000, 0x11)] pub set_counter_ch1: u32,
    #[entry(0x1601, 0x7010, 1)]    pub control_ch2: u32,
    #[entry(0x1601, 0x7010, 0x11)] pub set_counter_ch2: u32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A01, 0x1A03, 0x1A04, 0x1A08)]
#[pdos(2, Output, 0x1601, 0x1602, 0x1604)]
#[allow(non_camel_case_types)]
pub struct EL7031_Velocity {
    #[entry(0x1A01, 0x6000, 1)]  pub enc_status: u16,
    #[entry(0x1A01, 0x6000, 0x11)] pub enc_counter: u32,
    #[entry(0x1A01, 0x6000, 0x12)] pub enc_latch: u32,
    #[entry(0x1A03, 0x6010, 1)]  pub mot_status: u16,
    #[entry(0x1A04, 0x6010, 0x11)] pub info_data1: u16,
    #[entry(0x1A04, 0x6010, 0x12)] pub info_data2: u16,
    #[entry(0x1A08, 0x6010, 0x14)] pub mot_position: i32,

    #[entry(0x1601, 0x7000, 1)]  pub enc_control: u16,
    #[entry(0x1601, 0x7000, 0x11)] pub enc_set_counter: u32,
    #[entry(0x1602, 0x7010, 1)]  pub mot_control: u16,
    #[entry(0x1604, 0x7010, 0x21)] pub mot_velocity: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A01, 0x1A03, 0x1A04, 0x1A07)]
#[pdos(2, Output, 0x1601, 0x1602, 0x1604)]
#[allow(non_camel_case_types)]
pub struct EL7041_Velocity {
    #[entry(0x1A01, 0x6000, 1)]  pub enc_status: u16,
    #[entry(0x1A01, 0x6000, 0x11)] pub enc_counter: u32,
    #[entry(0x1A01, 0x6000, 0x12)] pub enc_latch: u32,
    #[entry(0x1A03, 0x6010, 1)]  pub mot_status: u16,
    #[entry(0x1A04, 0x6010, 0x11)] pub info_data1: u16,
    #[entry(0x1A04, 0x6010, 0x12)] pub info_data2: u16,
    #[entry(0x1A07, 0x6010, 0x14)] pub mot_position: i32,

    #[entry(0x1601, 0x7000, 1)]  pub enc_control: u16,
    #[entry(0x1601, 0x7000, 0x11)] pub enc_set_counter: u32,
    #[entry(0x1602, 0x7010, 1)]  pub mot_control: u16,
    #[entry(0x1604, 0x7010, 0x21)] pub mot_velocity: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A01, 0x1A03, 0x1A04, 0x1A08)]
#[pdos(2, Output, 0x1601, 0x1602, 0x1604)]
#[allow(non_camel_case_types)]
pub struct EL7047_Velocity {
    #[entry(0x1A01, 0x6000, 1)]  pub enc_status: u16,
    #[entry(0x1A01, 0x6000, 0x11)] pub enc_counter: u32,
    #[entry(0x1A01, 0x6000, 0x12)] pub enc_latch: u32,
    #[entry(0x1A03, 0x6010, 1)]  pub mot_status: u16,
    #[entry(0x1A04, 0x6010, 0x11)] pub info_data1: u16,
    #[entry(0x1A04, 0x6010, 0x12)] pub info_data2: u16,
    #[entry(0x1A08, 0x6010, 0x14)] pub mot_position: i32,

    #[entry(0x1601, 0x7000, 1)]  pub enc_control: u16,
    #[entry(0x1601, 0x7000, 0x11)] pub enc_set_counter: u32,
    #[entry(0x1602, 0x7010, 1)]  pub mot_control: u16,
    #[entry(0x1604, 0x7010, 0x21)] pub mot_velocity: i16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A01, 0x1A03, 0x1A04, 0x1A08)]
#[pdos(2, Output, 0x1601, 0x1602, 0x1603)]
#[allow(non_camel_case_types)]
pub struct EL7047_Position {
    #[entry(0x1A01, 0x6000, 1)]  pub enc_status: u16,
    #[entry(0x1A01, 0x6000, 11)] pub enc_counter: u32,
    #[entry(0x1A01, 0x6000, 12)] pub enc_latch: u32,
    #[entry(0x1A03, 0x6010, 1)]  pub mot_status: u16,
    #[entry(0x1A04, 0x6010, 11)] pub info_data1: u16,
    #[entry(0x1A04, 0x6010, 12)] pub info_data2: u16,
    #[entry(0x1A08, 0x6010, 14)] pub mot_position: i32,

    #[entry(0x1601, 0x7000, 1)]  pub enc_control: u16,
    #[entry(0x1601, 0x7000, 11)] pub enc_set_counter: u32,
    #[entry(0x1602, 0x7010, 1)]  pub mot_control: u16,
    #[entry(0x1603, 0x7010, 11)] pub mot_target: i32,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A01, 0x1A03, 0x1A07)]
#[pdos(2, Output, 0x1601, 0x1602, 0x1606)]
#[allow(non_camel_case_types)]
pub struct EL7047_Positioning {
    #[entry(0x1A01, 0x6000, 1)]  pub enc_status: u16,
    #[entry(0x1A01, 0x6000, 11)] pub enc_counter: u32,
    #[entry(0x1A01, 0x6000, 12)] pub enc_latch: u32,
    #[entry(0x1A03, 0x6010, 1)]  pub mot_status: u16,
    #[entry(0x1A07, 0x6020, 1)]  pub pos_status: u16,
    #[entry(0x1A07, 0x6020, 11)] pub act_pos: i32,
    #[entry(0x1A07, 0x6020, 21)] pub act_velo: u16,
    #[entry(0x1A07, 0x6020, 22)] pub drv_time: u32,

    #[entry(0x1601, 0x7000, 1)]  pub enc_control: u16,
    #[entry(0x1601, 0x7000, 11)] pub enc_set_counter: u32,
    #[entry(0x1602, 0x7010, 1)]  pub mot_control: u16,
    #[entry(0x1606, 0x7020, 1)]  pub pos_control: u16,
    #[entry(0x1606, 0x7020, 11)] pub target_pos: u32,
    #[entry(0x1606, 0x7020, 21)] pub target_velo: u16,
    #[entry(0x1606, 0x7020, 22)] pub start_type: u16,
    #[entry(0x1606, 0x7020, 23)] pub accel: u16,
    #[entry(0x1606, 0x7020, 24)] pub decel: u16,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A00, 0x1A01, 0x1A02, 0x1A03, 0x1A04, 0x1A05, 0x1A06, 0x1A0E)]
#[pdos(2, Output, 0x1600, 0x1601, 0x1608)]
#[dc(0x700, 2000000, 30000, 2000000, 1000)]
#[allow(non_camel_case_types)]
pub struct EL7211_0010_Velocity {
    #[entry(0x1A00, 0x6000, 0x11)] pub act_pos: u32,
    #[entry(0x1A01, 0x6010, 1)]    pub mot_status: u16,
    #[entry(0x1A02, 0x6010, 7)]    pub act_velo: i32,
    #[entry(0x1A03, 0x6010, 8)]    pub act_torq: i16,
    #[entry(0x1A04, 0x6010, 0x12)] pub info_data1: u16,
    #[entry(0x1A05, 0x6010, 0x13)] pub info_data2: u16,
    #[entry(0x1A06, 0x6010, 6)]    pub drag_error: i32,
    #[entry(0x1A0E, 0x6010, 3)]    pub mot_curr_mode: u8,

    #[entry(0x1600, 0x7010, 1)]   pub mot_control: u16,
    #[entry(0x1601, 0x7010, 6)]   pub target_velo: i32,
    #[entry(0x1608, 0x7010, 3)]   pub mot_mode: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
#[pdos(3, Input,  0x1A00, 0x1A01, 0x1A02, 0x1A04, 0x1A05, 0x1A06, 0x1A07, 0x1A0E)]
#[pdos(2, Output, 0x1600, 0x1601, 0x1608)]
#[dc(0x700, 2000000, 30000, 2000000, 1000)]
#[allow(non_camel_case_types)]
pub struct EL7221_9014_Velocity {
    #[entry(0x1A00, 0x6000, 0x11)] pub act_pos: u32,
    #[entry(0x1A01, 0x6010, 1)]    pub mot_status: u16,
    #[entry(0x1A02, 0x6010, 7)]    pub act_velo: i32,
    #[entry(0x1A04, 0x6010, 0x12)] pub info_data1: u16,
    #[entry(0x1A05, 0x6010, 0x13)] pub info_data2: u16,
    #[entry(0x1A06, 0x6010, 6)]    pub drag_error: i32,
    #[entry(0x1A07, 0x6010, 8)]    pub act_torq: i16,
    #[entry(0x1A0E, 0x6010, 3)]    pub mot_curr_mode: u8,

    #[entry(0x1600, 0x7010, 1)]   pub mot_control: u16,
    #[entry(0x1601, 0x7010, 6)]   pub target_velo: i32,
    #[entry(0x1608, 0x7010, 3)]   pub mot_mode: u8,
}

#[repr(C, packed)]
#[derive(SlaveProcessImage, Default)]
pub struct EL9505 {
}
