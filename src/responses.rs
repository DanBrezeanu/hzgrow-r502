use crate::utils::FromPayload;
use byteorder::{ByteOrder, BigEndian};

/// Responses to commands returned by the R502. Names are the same as commands.
#[derive(Debug)]
pub enum Reply {
    /// Contains system status and configuration information
    ReadSysPara(ReadSysParaResult),

    /// Contains result of password verification
    VfyPwd(VfyPwdResult),

    /// Contains result of acquiring an image
    GenImg(GenImgResult),
}

#[derive(Debug)]
pub struct ReadSysParaResult {
    pub address: u32,
    pub confirmation_code: u8,
    pub system_parameters: SystemParameters,
    pub checksum: u16,
}

impl FromPayload
for ReadSysParaResult {
    // Expected packet:
    // headr  | 0xEF 0x01 [2]
    // addr   | cmd.address [4]
    // ident  | 0x01 [1]
    // length | 0x00 0x03 [2] == 19 (3 + 16)
    // confrm | 0x0F [1]
    // params | (params) [16]
    // chksum | checksum [2]
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: payload[9],
            checksum: BigEndian::read_u16(&payload[26..28]),
            system_parameters: SystemParameters::from_payload(&payload[10..26]),
        };
    }
}

#[derive(Debug)]
pub struct VfyPwdResult {
    pub address: u32,
    /// Handshake result
    pub confirmation_code: PasswordVerificationState,
    pub checksum: u16,
}

impl FromPayload
for VfyPwdResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: PasswordVerificationState::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

#[derive(Debug)]
pub struct GenImgResult {
    pub address: u32,
    /// Fingerprint capture result
    pub confirmation_code: GenImgStatus,
    pub checksum: u16,
}

impl FromPayload
for GenImgResult {
    fn from_payload(payload: &[u8]) -> Self {
        return Self {
            address: BigEndian::read_u32(&payload[2..6]),
            confirmation_code: GenImgStatus::from(payload[9]),
            checksum: BigEndian::read_u16(&payload[10..12]),
        };
    }
}

/// System status and configuration.
#[derive(Debug)]
pub struct SystemParameters {
    /// Status information. Use instance methods of SystemParameters to get to individual bits.
    pub status_register: u16,

    /// System identifier code, whatever that means - datasheet says this has a constant value of
    /// 0x0009
    pub system_identifier_code: u16,

    /// Finger library size.
    pub finger_library_size: u16,

    /// Security level [1-5]
    pub security_level: u16,

    /// Device address, in case you forgot, but then you'd need the device address to send it the
    /// `ReadSysPara` command... 🤔
    pub device_address: u32,

    /// Packet size. Actually a size code [0-3]:\ 
    /// 0 = 32 bytes\ 
    /// 1 = 64 bytes\ 
    /// 2 = 128 bytes (the default)\ 
    /// 3 = 256 bytes
    pub packet_size: u16,

    /// Baud setting. To get actual baud value, multiply by 9600.
    ///
    /// Note, the datasheet contradicts itself as to what's the maximum baud rate supported by
    /// the device, and consequently what's the maximum here. In one place, it says the range is
    /// [1-6], in another it states the max baud rate is 115,200 giving [1-12].
    /// The default value is 6 for 57,600‬ baud.
    pub baud_setting: u16,
}

impl SystemParameters {
    /// True if the R502 is busy executing another command.
    ///
    /// *Busy* in the datasheet.
    pub fn busy(self) -> bool {
        return self.status_register & (1u16 << 0) != 0;
    }

    /// True if the module found a matching finger - however you should
    /// always check the response to the actual matching request.
    ///
    /// *Pass* in the datasheet.
    pub fn has_finger_match(self) -> bool {
        return self.status_register & (1u16 << 1) != 0;
    }

    /// True if the password given in the handshake is correct.
    ///
    /// *PWD* in the datasheet.
    pub fn password_ok(self) -> bool {
        return self.status_register & (1u16 << 2) != 0;
    }

    /// True if the image buffer contains a valid image.
    ///
    /// *ImgBufStat* in the datasheet.
    pub fn has_valid_image(self) -> bool {
        return self.status_register & (1u16 << 3) != 0;
    }
}

impl FromPayload
for SystemParameters {
    fn from_payload(payload: &[u8]) -> SystemParameters {
        // HZ R502's datasheet is a little inconsistent - sometimes the sizes are given in bytes
        // and sometimes in words; words are 16 bit (2 byte).
        // Pick a flipping unit and stick with it!
        SystemParameters {
            status_register: BigEndian::read_u16(&payload[0..2]),
            system_identifier_code: BigEndian::read_u16(&payload[2..4]),
            finger_library_size: BigEndian::read_u16(&payload[4..6]),
            security_level: BigEndian::read_u16(&payload[6..8]),
            device_address: BigEndian::read_u32(&payload[8..12]),
            packet_size: BigEndian::read_u16(&payload[12..14]),
            baud_setting: BigEndian::read_u16(&payload[12..16]),
        }
    }
}

/// Enum for the password handshake result
#[derive(Debug)]
pub enum PasswordVerificationState {
    Correct,
    Incorrect,
    Error,
}

impl PasswordVerificationState {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Correct,
            0x13 => Self::Incorrect,
            0x01 => Self::Error,
            _ => panic!("Invalid VfyPwdResult: {:02x}", byte),
        };
    }
}

#[derive(Debug)]
pub enum GenImgStatus {
    /// Fingerprint has been captured successfully
    Success,

    /// Error reading packet from the host
    PacketError,

    /// Finger not detected
    FingerNotDetected,

    /// Image failed to capture
    ImageNotCaptured,
}

impl GenImgStatus {
    pub fn from(byte: u8) -> Self {
        return match byte {
            0x00 => Self::Success,
            0x01 => Self::PacketError,
            0x02 => Self::FingerNotDetected,
            0x03 => Self::ImageNotCaptured,
            _ => panic!("Invalid GenImgStatus: {:02x}", byte),
        };
    }
}
