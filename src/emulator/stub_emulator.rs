use unicorn_engine::{
    ffi::uc_strerror,
    uc_error,
    unicorn_const::{Arch as UnicornArch, Mode as UnicornMode},
    HookType, Permission, RegisterX86, Unicorn,
};

use anyhow::Result;

use std::sync::{Arc, Mutex};

use crate::emulator::{
    config::{Config, Section},
    packet::{PathPacket, WardSpawnPacket},
};

pub struct StubEmulator<'a> {
    config: Config,

    uc: unicorn_engine::Unicorn<'a, ()>,

    packet_addr: u64,
    packet_size: usize,
}

impl<'a> StubEmulator<'a> {
    const PAGE_SIZE: usize = 0x1000;

    const STACK_BASE: u64 = 0x7FFFFFFF0000;
    const STACK_SIZE: usize = 0x2000;

    const HEAP_BASE: u64 = 0x7FFFFFFF8000;
    const HEAP_SIZE: usize = 0x2000;

    const HEAP_CURSOR_PTR: u64 = 0x0;

    pub fn new(config: Config) -> Self {
        let uc = Unicorn::new(UnicornArch::X86, UnicornMode::MODE_64)
            .expect("Failed to create unicorn.");

        Self {
            config,

            uc,

            packet_addr: 0,
            packet_size: 0,
        }
    }

    pub fn setup(&mut self) -> Result<()> {
        self.map_stack()?;
        self.map_heap()?;
        self.map_sections()?;

        self.patch_functions();

        Ok(())
    }

    pub fn setup_args(&mut self, payload: &[u8]) -> Result<()> {
        self.packet_size = 0x90;
        self.packet_addr = self.alloc(self.packet_size);

        let ptr = self.alloc_and_store(payload)?;
        let payload_ptr = self.alloc_and_store(&ptr.to_le_bytes())?;
        let payload_end = ptr + payload.len() as u64;

        self.write_reg(RegisterX86::RCX, self.packet_addr)?; // Arg1
        self.write_reg(RegisterX86::RDX, payload_ptr)?; // Arg2
        self.write_reg(RegisterX86::R8, payload_end)?; // Arg3

        Ok(())
    }

    pub fn reset(&mut self) {
        self.set_heap_cursor(0);

        self.write_reg(
            RegisterX86::RSP,
            Self::STACK_BASE + (Self::STACK_SIZE - 0x100) as u64,
        )
        .unwrap();
    }

    pub fn call_decrypt_ward_spawn_packet(
        &mut self,
        call_rva: u64,
        end_rva: u64,
        timestamp: f32,
    ) -> Result<WardSpawnPacket> {
        let packet_addr_clone = self.packet_addr;

        let x = Arc::new(Mutex::new(0.0f32));
        let y = Arc::new(Mutex::new(0.0f32));

        let x_clone = Arc::clone(&x);
        let y_clone = Arc::clone(&y);

        let id = Arc::new(Mutex::new(0u32));
        let id_clone = Arc::clone(&id);

        let owner_id = Arc::new(Mutex::new(0u32));
        let owner_id_clone = Arc::clone(&owner_id);

        let x_offset = self.config.ward_spawn_decrypt.x_offset as u16;
        let x_write_count = self.config.ward_spawn_decrypt.x_write_count as usize;

        let y_offset = self.config.ward_spawn_decrypt.y_offset as u16;
        let y_write_count = self.config.ward_spawn_decrypt.y_write_count as usize;

        let id_offset = self.config.ward_spawn_decrypt.id_offset as u16;
        let owner_id_offset = self.config.ward_spawn_decrypt.owner_id_offset as u16;

        let mut packet_write_count: Vec<usize> = vec![0; self.packet_size];

        /*
        "name_offset": "0x60",
        "name_len_offset": "0x68",
        */

        self.uc
            .add_mem_hook(
                HookType::MEM_WRITE,
                self.packet_addr,
                self.packet_addr + self.packet_size as u64,
                move |_, _, addr, size, value| {
                    if size == 1 {
                        return false;
                    }

                    let offset = (addr - packet_addr_clone) as u16;

                    let count = packet_write_count[offset as usize];

                    if offset == x_offset && count == x_write_count {
                        let mut x = x_clone.lock().unwrap();
                        *x = f32::from_bits(value as u32);
                    }

                    if offset == y_offset && count == y_write_count {
                        let mut y = y_clone.lock().unwrap();
                        *y = f32::from_bits(value as u32);
                    }
                    
                    // for these two, we only care about the first write
                    if offset == id_offset && count == 0 {
                        let mut id = id_clone.lock().unwrap();
                        *id = value as u32;
                    }

                    if offset == owner_id_offset && count == 0 {
                        let mut owner_id = owner_id_clone.lock().unwrap();
                        *owner_id = value as u32;
                    }

                    for i in offset..offset + size as u16 {
                        if i < packet_write_count.len() as u16 {
                            packet_write_count[i as usize] += 1;
                        }
                    }

                    if size > 1 || count == 0 {
                        return true;
                    }

                    false
                },
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to add packet write hook: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        let _ = self.uc.emu_start(
            self.rva_to_address(call_rva),
            self.rva_to_address(end_rva),
            0,
            0,
        );

        let x = x.lock().unwrap();
        let y = y.lock().unwrap();

        let id = id.lock().unwrap();
        let owner_id = owner_id.lock().unwrap();

        let ptr = self
            .read_ptr_on(self.packet_addr + self.config.ward_spawn_decrypt.name_offset)
            .unwrap();
        let size = self
            .read_u32_on(self.packet_addr + self.config.ward_spawn_decrypt.name_len_offset)
            .unwrap();

        let name = self.read_str_on(ptr, size as usize).unwrap();

        Ok({
            WardSpawnPacket {
                timestamp,
                name,
                id: *id,
                owner_id: *owner_id,
                x: *x as i32,
                y: *y as i32,
            }
        })
    }

    pub fn call_decrypt_pos_packet(
        &mut self,
        call_rva: u64,
        end_rva: u64,
        timestamp: f32,
    ) -> Result<PathPacket> {
        self.uc
            .reg_write(
                RegisterX86::RSP,
                Self::STACK_BASE + (Self::STACK_SIZE as u64 - 0x100),
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to write stack pointer: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        let _ = self.uc.emu_start(
            self.rva_to_address(call_rva),
            self.rva_to_address(end_rva),
            0,
            0,
        );

        let size = self
            .read_u32_on(self.packet_addr + self.config.mov_decrypt.payload_size_offset)
            .unwrap();
        let ptr = self
            .read_ptr_on(self.packet_addr + self.config.mov_decrypt.payload_offset)
            .unwrap();

        let payload = self.read_buffer_on(ptr, size as usize)?;

        let packet = PathPacket::parse(timestamp, payload)?;

        Ok(packet)
    }

    fn map_stack(&mut self) -> Result<()> {
        self.uc
            .mem_map(
                Self::STACK_BASE,
                Self::STACK_SIZE,
                Permission::READ | Permission::WRITE,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to map stack: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        self.uc
            .reg_write(
                RegisterX86::RSP,
                Self::STACK_BASE + (Self::STACK_SIZE as u64 - 0x100),
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to write stack pointer: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        Ok(())
    }

    fn map_heap(&mut self) -> Result<()> {
        self.uc
            .mem_map(
                Self::HEAP_BASE,
                Self::HEAP_SIZE,
                Permission::READ | Permission::WRITE,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to map heap: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        self.uc
            .mem_map(
                Self::align_addr(self.rva_to_address(Self::HEAP_CURSOR_PTR)),
                Self::align_size(4),
                Permission::READ | Permission::WRITE,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to map heap cursor pointer: {}",
                    Self::uc_err_to_str(e)
                )
            })?;

        self.set_heap_cursor(0);

        Ok(())
    }

    fn map_sections(&mut self) -> Result<()> {
        self.map_section(self.config.text.clone())?;
        self.map_section(self.config.data.clone())?;
        self.map_section(self.config.rdata.clone())?;
        Ok(())
    }

    fn map_section(&mut self, sect: Arc<Section>) -> Result<()> {
        let sect_addr = self.rva_to_address(sect.rva);
        let sect_size = sect.size;

        self.uc
            .mem_map(
                Self::align_addr(sect_addr),
                Self::align_size(sect_size as usize),
                Permission::READ | Permission::WRITE | Permission::EXEC,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "[SETUP ERROR] Failed to map .{} section: {}",
                    sect.name,
                    Self::uc_err_to_str(e)
                )
            })?;

        self.uc.mem_write(sect_addr, &sect.raw).map_err(|e| {
            anyhow::anyhow!(
                "[SETUP ERROR] Failed to write .{} section: {}",
                sect.name,
                Self::uc_err_to_str(e)
            )
        })?;

        Ok(())
    }

    fn patch_functions(&mut self) {
        let patch1 = [0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00, 0xC3];

        self.uc
            .mem_write(self.rva_to_address(self.config.skip), &patch1)
            .unwrap();

        let patch2 = [
            0x53, 0x57, 0x56, 0x55, 0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53, 0x41, 0x54,
            0x41, 0x55, 0x41, 0x56, 0x41, 0x57, 0x48, 0xB8, 0x00, 0x00, 0xFD, 0x6A, 0xF7, 0x7F,
            0x00, 0x00, 0x48, 0x8B, 0x18, 0x48, 0xB8, 0x00, 0x80, 0xFF, 0xFF, 0xFF, 0x7F, 0x00,
            0x00, 0x48, 0x8D, 0x04, 0x18, 0x48, 0x89, 0x01, 0x89, 0x51, 0x08, 0x01, 0xD3, 0x48,
            0xB8, 0x00, 0x00, 0xFD, 0x6A, 0xF7, 0x7F, 0x00, 0x00, 0x89, 0x18, 0x41, 0x5F, 0x41,
            0x5E, 0x41, 0x5D, 0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A, 0x41, 0x59, 0x41, 0x58, 0x5D,
            0x5E, 0x5F, 0x5B, 0xC3,
        ];

        self.uc
            .mem_write(self.rva_to_address(self.config.alloc1), &patch2)
            .unwrap();

        self.uc
            .mem_write(self.rva_to_address(self.config.alloc2), &patch2)
            .unwrap();
    }

    pub fn get_heap_cursor(&mut self) -> u64 {
        let offset = self.rva_to_address(Self::HEAP_CURSOR_PTR);
        let mut buffer = [0u8; 8];
        self.uc
            .mem_read(offset, &mut buffer)
            .expect("Failed to read memory");
        u64::from_le_bytes(buffer)
    }

    pub fn set_heap_cursor(&mut self, heap_cursor: u64) {
        let offset = self.rva_to_address(Self::HEAP_CURSOR_PTR);
        self.uc
            .mem_write(offset, &heap_cursor.to_le_bytes())
            .expect("Failed to write memory");
    }

    fn alloc(&mut self, size: usize) -> u64 {
        let heap_cursor = self.get_heap_cursor();
        let ptr = Self::HEAP_BASE + heap_cursor;
        self.set_heap_cursor(heap_cursor + size as u64);
        ptr
    }

    fn alloc_and_store(&mut self, data: &[u8]) -> Result<u64> {
        let ptr = self.alloc(data.len());
        self.uc.mem_write(ptr, data).map_err(|e| {
            anyhow::anyhow!(
                "[SETUP ERROR] Failed to store on heap: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(ptr)
    }

    fn write_reg(&mut self, reg: RegisterX86, value: u64) -> Result<()> {
        self.uc.reg_write(reg, value).map_err(|e| {
            anyhow::anyhow!(
                "[SETUP ERROR] Failed to write register: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(())
    }

    fn read_str_on(&self, addr: u64, size: usize) -> Result<String> {
        let mut buffer = vec![0u8; size];
        self.uc.mem_read(addr, &mut buffer).map_err(|e| {
            anyhow::anyhow!(
                "[RUNTIME ERROR] Failed to read string: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(String::from_utf8(buffer).unwrap())
    }

    fn read_buffer_on(&self, addr: u64, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        self.uc.mem_read(addr, &mut buffer).map_err(|e| {
            anyhow::anyhow!(
                "[RUNTIME ERROR] Failed to read buffer: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(buffer)
    }

    fn read_u32_on(&self, addr: u64) -> Result<u32> {
        let mut buffer = [0u8; 4];
        self.uc.mem_read(addr, &mut buffer).map_err(|e| {
            anyhow::anyhow!(
                "[RUNTIME ERROR] Failed to read u32: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_ptr_on(&self, addr: u64) -> Result<u64> {
        let mut buffer = [0u8; 8];
        self.uc.mem_read(addr, &mut buffer).map_err(|e| {
            anyhow::anyhow!(
                "[RUNTIME ERROR] Failed to read ptr: {}",
                Self::uc_err_to_str(e)
            )
        })?;
        Ok(u64::from_le_bytes(buffer))
    }

    fn rva_to_address(&self, rva: u64) -> u64 {
        rva + self.config.base_addr
    }

    fn address_to_rva(&self, addr: u64) -> u64 {
        addr - self.config.base_addr
    }

    fn align_addr(addr: u64) -> u64 {
        addr & !(Self::PAGE_SIZE as u64 - 1)
    }

    fn align_size(size: usize) -> usize {
        (size + Self::PAGE_SIZE - 1) & !(Self::PAGE_SIZE - 1)
    }

    fn uc_err_to_str(err: uc_error) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(uc_strerror(err))
                .to_string_lossy()
                .to_string()
        }
    }
}
