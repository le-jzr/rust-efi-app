use efi_types;

use protocol::{ Color, Guid, Status, Result, status_to_result, status_to_status };

use core::convert::TryFrom;
use core::fmt;

/// This protocol is used to control text-based output devices.
pub struct Protocol {
    interface: efi_types::SIMPLE_TEXT_OUTPUT_INTERFACE,
}

pub type Column = i32;
pub type Row = i32;
pub type ModeNumber = i32;

#[derive(Copy, Clone, Debug)]
pub struct Attribute {
    code: u32,
}

impl Attribute {
    #[inline]
    pub fn new(foreground: Color, background: Color) -> Self {
        Attribute{ code: ((foreground as u8) | (((background as u8) & 0x07) << 4)).into() }
    }

    #[inline]
    pub fn foreground(self) -> Color {
        Color::try_from((self.code & 0x0f) as u8).unwrap()
    }

    #[inline]
    pub fn background(self) -> Color {
        Color::try_from(((self.code >> 4) & 0x07) as u8).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Mode {
    // Note: This struct isn't used for FFI directly due to
    // bool not having stable representation.

    pub max_mode: ModeNumber,

    // current settings
    pub mode: ModeNumber,
    pub attribute: Attribute,
    pub cursor_column: Column,
    pub cursor_row: Row,
    pub cursor_visible: bool,
}

impl Protocol {
    pub const GUID: Guid = Guid(0x387477c2,0x69c7,0x11d2,[0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b]);

    pub fn mode(&mut self) -> Mode {
        let m = unsafe {&*self.interface.Mode};
        Mode {
            max_mode: m.MaxMode as _,
            mode: m.Mode as _,
            attribute: Attribute{ code: m.Attribute as _ },
            cursor_column: m.CursorColumn as _,
            cursor_row: m.CursorRow as _,
            cursor_visible: m.CursorVisible != 0,
        }
    }

    /// Resets the text output device hardware.
    ///
    /// ```text
    ///     The `reset()` function resets the text output device
    ///     hardware. The cursor position is set to `(0, 0)`,
    ///     and the screen is cleared to the default background color for the output device.
    ///
    ///     As part of initialization process, the firmware/device will make
    ///     a quick but reasonable attempt to verify that the device is functioning.
    ///     If the `extended_verification` flag is `true` the firmware may take
    ///     an extended amount of time to verify the device is operating on
    ///     reset. Otherwise the reset operation is to occur as quickly as possible.
    ///
    ///     The hardware verification process is not defined
    ///     by this specification and is left up to the
    ///     platform firmware or driver to implement.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The text output device is not functioning correctly and could not be reset.
    ///
	pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
	    let func = self.interface.Reset.unwrap();
	    let status = unsafe { func(&mut self.interface, extended_verification as u8) };
	    status_to_result(status, ())
	}

    /// Writes a string to the output device. `string` must be zero-terminated.
    ///
    /// ```text
    ///     The `output_string()` function writes a string to the output device.
    ///     This is the most basic output mechanism on an output device.
    ///     The string is displayed at the current cursor location on
    ///     the output device(s) and the cursor is advanced according to the
    ///     rules listed in Table 103:
    ///
    ///     +----------+---------+------------------------------------------------------+
    ///     | Mnemonic | Unicode | Description                                          |
    ///     +----------+---------+------------------------------------------------------+
    ///     | Null     | U+0000  | Ignore the character, and do not move the cursor.    |
    ///     +----------+---------+------------------------------------------------------+
    ///     | BS       | U+0008  | If the cursor is not at the left edge of the         |
    ///     |          |         | display, then move the cursor left one column.       |
    ///     +----------+---------+------------------------------------------------------+
    ///     | LF       | U+000A  | If the cursor is at the bottom of the display,       |
    ///     |          |         | then scroll the display one row, and do not          |
    ///     |          |         | update the cursor position.                          |
    ///     |          |         | Otherwise, move the cursor down one row.             |
    ///     +----------+---------+------------------------------------------------------+
    ///     | CR       | U+000D  | Move the cursor to the beginning of the current row. |
    ///     +----------+---------+------------------------------------------------------+
    ///     | Other    | U+XXXX  | Print the character at the current cursor position   |
    ///     |          |         | and move the cursor right one column. If this moves  |
    ///     |          |         | the cursor past the right edge of the display,       |
    ///     |          |         | then the line should wrap to the beginning of the    |
    ///     |          |         | next line. This is equivalent to inserting a CR      |
    ///     |          |         | and an LF. Note that if the cursor is at the bottom  |
    ///     |          |         | of the display, and the line wraps, then the display |
    ///     |          |         | will be scrolled one line.                           |
    ///     +----------+---------+------------------------------------------------------+
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device reported an error while attempting to output the text.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The output device’s mode is not currently in a defined text mode.
    ///
    /// * `EFI_WARN_UNKNOWN_GLYPH`
    ///     * This warning code indicates that some of the characters
    ///     in the string could not be rendered and were skipped.
    ///
    pub fn output_string_utf16(&mut self, string: &[u16]) -> Result<Status> {
        assert!(string[string.len()-1] == 0);

	    let func = self.interface.OutputString.unwrap();
	    let status = unsafe { func(&mut self.interface, string.as_ptr() as *mut u16) };
	    status_to_status(status)
    }

    /// Verifies that all characters in a string can be output to the target device.
    ///
    /// ```text
    ///     The `test_string()` function verifies that all characters in
    ///     a string can be output to the target device.
    ///     This function provides a way to know if the desired character
    ///     codes are supported for rendering on the output device(s).
    ///     This allows the installation procedure (or EFI image) to
    ///     at least select character codes that the output devices are
    ///     capable of displaying. Since the output device(s) may be
    ///     changed between boots, if the loader cannot adapt to such
    ///     changes it is recommended that the loader call `output_string()`
    ///     with the text it has and ignore any “unsupported” error codes.
    ///     Devices that are capable of displaying the Unicode
    ///     character codes will do so.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * Some of the characters in the string cannot be rendered by one or
    ///     more of the output devices mapped by the EFI handle.
    ///
    pub fn test_string_utf16(&mut self, string: &[u16]) -> Result<()> {
        assert!(string[string.len()-1] == 0);

        unimplemented!()
    }

    /// Returns information for an available text mode that the output device(s) supports.
    ///
    /// On success, returns a pair `(number_of_columns, number_of_rows)`.
    ///
    /// ```text
    ///     The `query_mode()` function returns information for an available
    ///     text mode that the output device(s) supports.
    ///     It is required that all output devices support at least 80x25 text mode.
    ///     This mode is defined to be mode 0. If the output devices support 80x50,
    ///     that is defined to be mode 1.
    ///     All other text dimensions supported by the device will follow as
    ///     modes 2 and above. If an output device supports modes 2 and above,
    ///     but does not support 80x50, then querying for mode 1 will return
    ///     `EFI_UNSUPPORTED`.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The mode number was not valid.
    ///
    pub fn query_mode(&mut self, mode_number: ModeNumber) -> Result<(Column, Row)> {
        unimplemented!()
    }

    /// Sets the output device(s) to a specified mode.
    ///
    /// ```text
    ///     The `set_mode()` function sets the output device(s) to the requested mode.
    ///     On success the device is in the geometry for the requested mode,
    ///     and the device has been cleared to the current background color
    ///     with the cursor at `(0, 0)`.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The mode number was not valid.
    ///
    pub fn set_mode(&mut self, mode_number: ModeNumber) -> Result<()> {
        unimplemented!()
    }

    /// Sets the background and foreground colors for the `output_string()'
    /// and `clear_screen()` functions.
    ///
    /// ```text
    ///     The `set_attribute()` function sets the background and foreground
    ///     colors for the `output_string()` and `clear_screen()` functions.
    ///     The color mask can be set even when the device is in an invalid text mode.
    ///     Devices supporting a different number of text colors are required to emulate the above
    ///     colors to the best of the device’s capabilities.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request.
    ///
    pub fn set_attribute(&mut self, attr: Attribute) -> Result<()> {
        unimplemented!()
    }

    ///
    /// ```text
    ///     The `clear_screen()` function clears the output device(s) display
    ///     to the currently selected background color.
    ///     The cursor position is set to `(0, 0)`.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The output device is not in a valid text mode.
    ///
    pub fn clear_screen(&mut self) -> Result<()> {
        let func = self.interface.ClearScreen.unwrap();
	    let status = unsafe { func(&mut self.interface) };
	    status_to_result(status, ())
    }

    /// Sets the current coordinates of the cursor position.
    ///
    /// ```text
    ///     The `set_cursor_position()` function sets the current coordinates
    ///     of the cursor position. The upper left corner of the screen
    ///     is defined as coordinate `(0, 0)`.
    /// ```
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The output device is not in a valid text mode,
    ///     or the cursor position is invalid for the current mode.
    ///
    pub fn set_cursor_position(&mut self, column: Column, row: Row) -> Result<()> {
        unimplemented!()
    }

    /// Makes the cursor visible or invisible.
    ///
    /// **Errors**
    ///
    /// * `EFI_DEVICE_ERROR`
    ///     * The device had an error and could not complete the request
    ///     or the device does not support changing the cursor mode.
    ///
    /// * `EFI_UNSUPPORTED`
    ///     * The output device does not support visibility control of the cursor.
    ///
    pub fn enable_cursor(&mut self, visible_cursor: bool) -> Result<()> {
        unimplemented!()
    }

    /// Writes a string to the output device.
    ///
    /// This is a convenience method that wraps `output_string_utf16`.
    ///
    pub fn output_string(&mut self, s: &str) -> Result<Status> {

        let buffer = &mut [0_u16; 32];
        let mut i = 0;
        let mut stat = Status::success();

        for c in s.chars() {
            if i >= buffer.len() - 2 {
                buffer[i] = 0;
                let status = try!(self.output_string_utf16(&buffer[..i+1]));
                if !status.is_success() {
                    stat = status;
                }
                i = 0;
            }

            let code = c as u32;

            // Expand newline.
            if code == '\n' as u32 {
                buffer[i] = '\r' as u16;
                buffer[i+1] = '\n' as u16;
                i += 2;
                continue;
            }

            if code >= 0xd800 && code < 0xe000 {
                // Illegal code points.
                buffer[i] = 0xfffd;
                i += 1;
                continue;
            }

            if code < 0x10000 {
                buffer[i] = code as u16;
                i += 1;
            } else {
                // Emit surrogates.
                let code = code - 0x10000;
                let high_surrogate = 0xd800 + code >> 10;
                let low_surrogate = 0xdc00 + code & 0x03ff;
                buffer[i] = high_surrogate as u16;
                buffer[i+1] = low_surrogate as u16;
                i += 2;
            }
        }

        buffer[i] = 0;
        let status = try!(self.output_string_utf16(&buffer[..i+1]));
        if !status.is_success() {
            stat = status
        }

        Ok(stat)
    }

}

impl fmt::Write for Protocol {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.output_string(s) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
