use max78000_hal::error::{ErrorKind, Result};

static mut FLASH: Option<FlashEntry> = None;

#[repr(C)]
#[derive(Debug)]
struct FlashEntry {
    flash_magic: u32,
    component_count: u32,
    component_ids: [u32; 32],
}

extern "C" {
    fn init_flash(magic: u32) -> i32;
    fn read_flash() -> FlashEntry;
    fn write_flash(entry: &FlashEntry);
    fn poll_flash() -> i32;
}

pub fn init(magic: u32) -> Result<()> {
    let result = unsafe { init_flash(magic) };
    match result {
        0 => {
            unsafe {
                let entry = read_flash();
                FLASH = Some(match poll_flash() {
                    0 => Ok(entry),
                    1 => Err(ErrorKind::Fail),
                    _ => unreachable!(),
                }?)
            }
            Ok(())
        }
        1 => Err(ErrorKind::Fail),
        _ => unreachable!(),
    }
}

pub fn get_component_ids() -> Result<&'static [u32]> {
    unsafe {
        FLASH
            .as_ref()
            .map(|flash| &flash.component_ids[..flash.component_count as usize])
            .ok_or(ErrorKind::Uninitialized)
    }
}

pub fn swap_component(id_old: u32, id_new: u32) -> Result<()> {
    unsafe {
        FLASH
            .as_mut()
            .map(|flash| {
                *flash.component_ids.iter_mut().find(|x| **x == id_old)? = id_new;
                Some(())
            })
            .ok_or(ErrorKind::Uninitialized)?
            .ok_or(ErrorKind::BadParam)?;
        write_flash(FLASH.as_ref().unwrap());
        match poll_flash() {
            0 => Ok(()),
            1 => Err(ErrorKind::Fail),
            _ => unreachable!(),
        }
    }
}
