
pub const PS_MOVE_VID: u16 = 0x054c;
pub const PS_MOVE_PID: u16 = 0x03d5; // PSMove ZCM1
pub const PSMOVE_BTADDR_GET_ZCM1_SIZE: usize = 16;
pub const PSMOVE_BTADDR_GET_ZCM2_SIZE: usize = 21;
pub const PSMOVE_BTADDR_GET_MAX_SIZE: usize = PSMOVE_BTADDR_GET_ZCM2_SIZE;

pub enum PSMoveRequestType {
    GetBTAddr = 0x04,
}