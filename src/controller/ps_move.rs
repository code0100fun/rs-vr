
const PS_MOVE_VID: u16 = 0x054c;
const PS_MOVE_PID: u16 = 0x03d5; // PSMove ZCM1
const PSMOVE_BTADDR_GET_ZCM1_SIZE: usize = 16;
const PSMOVE_BTADDR_GET_ZCM2_SIZE: usize = 21;
const PSMOVE_BTADDR_GET_MAX_SIZE: usize = PSMOVE_BTADDR_GET_ZCM2_SIZE;

enum PSMoveRequestType {
    GetBTAddr = 0x04,
}